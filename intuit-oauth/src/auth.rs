use oauth2::{CsrfToken, ClientId, ClientSecret, RedirectUrl};
use reqwest::{Client, header, StatusCode};
use serde::{Deserialize, Serialize};

pub const AUTH_ENDPOINT: &'static str = "https://appcenter.intuit.com/connect/oauth2";
pub const TOKEN_ENDPOINT: &'static str = "https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer";
pub const REVOKE_ENDPOINT: &'static str = "https://developer.api.intuit.com/v2/oauth2/tokens/revoke";
pub const ACCOUNTING_SCOPE: &'static str = "com.intuit.quickbooks.accounting";

pub enum Environment {
    PRODUCTION,
    SANDBOX
}

impl Environment {
    pub fn discovery_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer.intuit.com/.well-known/openid_configuration/",
            Environment::SANDBOX => "https://developer.intuit.com/.well-known/openid_sandbox_configuration/",
        }
    }
    pub fn migration_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer-sandbox.api.intuit.com/v2/oauth2/tokens/migrate",
            Environment::SANDBOX => "https://developer.api.intuit.com/v2/oauth2/tokens/migrate",
        }
    }
    pub fn user_info_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://accounts.platform.intuit.com/v1/openid_connect/userinfo",
            Environment::SANDBOX => "https://sandbox-accounts.platform.intuit.com/v1/openid_connect/userinfo",
        }
    }
}

#[derive(Deserialize, Debug)]
struct DiscoveryDoc {
    #[serde(default)]
    issuer: String,
    #[serde(default)]
    authorization_endpoint: String,
    #[serde(default)]
    token_endpoint: String,
    #[serde(default)]
    userinfo_endpoint: String,
    #[serde(default)]
    revocation_endpoint: String,
    #[serde(default)]
    jwks_uri: String,
    #[serde(default)]
    response_types_supported: Vec<String>,
    #[serde(default)]
    subject_types_supported: Vec<String>,
    #[serde(default)]
    id_token_signing_alg_values_supported: Vec<String>,
    #[serde(default)]
    scopes_supported: Vec<String>,
    #[serde(default)]
    token_endpoint_auth_methods_supported: Vec<String>,
    #[serde(default)]
    claims_supported: Vec<String>
}

pub struct Unauthorized;
pub struct Authorized {
    state_token: CsrfToken,
    discovery_doc: DiscoveryDoc,
    // self.realm_id = realm_id
    //     self.access_token = access_token
    //     self.expires_in = None
    //     self.refresh_token = refresh_token
    //     self.x_refresh_token_expires_in = None
    //     self.id_token = id_token
    pub access_token: AccessToken,
    id_token: String,
    client: Client
}
pub trait AuthorizeType {}
impl AuthorizeType for Authorized {}
impl AuthorizeType for Unauthorized {}

pub(crate) struct AuthClient<T>
where T: AuthorizeType
{
    client_id: String,
    client_secret: String,
    realm_id: String,
    redirect_uri: String,
    environment: Environment,
    pub data: T
}

impl<T> AuthClient<T> 
where T: AuthorizeType
{
    async fn get_discovery_doc(&self) -> DiscoveryDoc {
        let url = self.environment.discovery_url();
        let resp = reqwest::get(url).await.expect("Error getting discovery doc from url");
        if !resp.status().is_success() {
            panic!("Error getting discovery doc: {}", resp.status())
        }
        match resp.json().await {
            Ok(doc) => {
                println!("{doc:?}");
                doc
            },
            Err(e) => panic!("Error deseralizing discovery doc: {e}"),
        }
    }

    async fn get_access_token(&mut self, auth_code: &str) -> Result<AccessToken, APIError> {
        let mut headers = header::HeaderMap::new();
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let params = [
            ("grant_type", "authorization_code"),
            ("code", auth_code),
            ("redirect_uri", &self.redirect_uri),
        ];
        let client = reqwest::Client::new();
        let resp = client
            .post("https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer")
            .headers(headers)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await.unwrap();

        if !resp.status().is_success() {
            return Err(APIError{status_code: resp.status(), body: resp.text().await.unwrap()});
        }

        // Unwrap the response.
        let t: AccessToken = resp.json().await.unwrap();
        println!("{t:?}");
        Ok(t)
    }
}

impl AuthClient<Unauthorized> {

    pub fn new<X, R, Q, O>(client_id: X, client_secret: Q, redirect_uri: R, realm_id: O, environment: Environment) -> Self
    where X: ToString, Q: ToString, R: ToString, O: ToString
    {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
            realm_id: realm_id.to_string(),
            environment,
            data: Unauthorized
        }
    }

    pub async fn authorize(mut self, auth_code: &str) -> AuthClient<Authorized> {
        let doc = self.get_discovery_doc().await;
        let tokens = self.get_access_token(auth_code).await.unwrap();

        let data = Authorized {
            state_token: CsrfToken::new_random(),
            discovery_doc: doc,
            access_token: tokens,
            id_token: String::new(),
            client: Client::new(),
        };

        AuthClient { 
            client_id: self.client_id,
            client_secret: self.client_secret,
            realm_id: self.realm_id, 
            redirect_uri: self.redirect_uri, 
            environment: self.environment,
            data
        }
    }
}

impl AuthClient<Authorized> {
    pub async fn refresh_access_token(&mut self) -> AccessToken {
        let mut headers = header::HeaderMap::new();
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.data.access_token.refresh_token),
        ];
        let resp = self.data.client
            .post("https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer")
            .headers(headers)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await
            .unwrap();

        let t: AccessToken = resp.json().await.unwrap();

        self.data.access_token = t.clone();
        t
    }
}
