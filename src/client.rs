use base64::Engine;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue},
    Client, Method, Request,
};
use serde::{Deserialize, Serialize};
// use tokio::sync::Semaphore;
use url::Url;

use crate::{error::APIError, DiscoveryDoc, Environment};

#[derive(Serialize)]
pub struct QBContext {
    pub(crate) environment: Environment,
    pub(crate) company_id: String,
    pub(crate) access_token: String,
    pub(crate) expires_in: DateTime<Utc>,
    // TODO Check if this should be in an option
    pub(crate) refresh_token: Option<String>,
    pub(crate) discovery_doc: DiscoveryDoc,
    // #[serde(skip)]
    // limiter: Semaphore,
}

#[derive(Serialize, Deserialize)]
struct AuthTokenResponse {
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    x_refresh_token_expires_in: u64,
    access_token: String,
}

impl QBContext {
    /// Creates a new `QBContext` with the given parameters
    pub async fn new(
        environment: Environment,
        company_id: String,
        access_token: String,
        refresh_token: Option<String>,
        client: &Client,
    ) -> Result<Self, APIError> {
        Ok(Self {
            environment,
            company_id,
            access_token,
            expires_in: Utc::now() + chrono::Duration::hours(999),
            refresh_token,
            discovery_doc: DiscoveryDoc::get(environment, client).await?,
            limiter: Semaphore::new(1),
        })
    }

    pub async fn new_from_env(
        environment: Environment,
        client: &Client,
    ) -> Result<Self, APIError> {
        let company_id = std::env::var("QB_COMPANY_ID")?;
        let access_token = std::env::var("QB_ACCESS_TOKEN")?;
        let refresh_token = std::env::var("QB_REFRESH_TOKEN").ok();
        let context =
            Self::new(environment, company_id, access_token, refresh_token, client).await?;
        Ok(context)
    }

    /// Checks if the current context is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_in
    }

    /// Refreshes the `access_token`, does not check if it's expired before it does so
    pub async fn refresh_access_token(
        &mut self,
        client_id: &str,
        client_secret: &str,
        client: &Client,
    ) -> Result<(), APIError> {
        // TODO Use types to prevent this from happening
        let Some(refresh_token) = self.refresh_token.as_deref() else {
            return Err(APIError::NoRefreshToken);
        };

        let auth_string = format!("{client_id}:{client_secret}");
        let auth_string = base64::engine::general_purpose::STANDARD.encode(auth_string);

        let request = client
            .request(Method::POST, &self.discovery_doc.token_endpoint)
            .bearer_auth(auth_string)
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=refresh_token&refresh_token={refresh_token}"
            ))
            .build()?;

        let response = client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
        }

        let AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in,
            ..
        } = response.json().await?;

        self.refresh_token = Some(refresh_token);
        self.access_token = access_token;
        self.expires_in = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        Ok(())
    }

    pub async fn check_authorized(&self, client: &Client) -> Result<bool, APIError> {
        let request = client
            .request(Method::GET, self.environment.user_info_url())
            .bearer_auth(&self.access_token)
            .header("ACCEPT", "application/json")
            .build()?;
        let response = client.execute(request).await?;
        let status = response.status();
        if !status.is_success() {
            println!(
                "Failed to check authorized status: {} - {}",
                status,
                response.text().await?
            );
        }
        Ok(status.is_success())
    }
}

pub(crate) fn build_headers(
    content_type: &str,
    access_token: &str,
) -> Result<HeaderMap, InvalidHeaderValue> {
    let bt = format!("Bearer {access_token}");
    let bearer =
        header::HeaderValue::from_str(&bt).expect("Invalid access token in Authorized Client");
    let mut headers = header::HeaderMap::new();
    headers.append(header::AUTHORIZATION, bearer);
    if content_type != "multipart/form-data" {
        headers.append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(content_type)?,
        );
    }
    headers.append(
        header::ACCEPT,
        header::HeaderValue::from_str("application/json")?,
    );
    Ok(headers)
}

pub(crate) fn build_request<B: serde::Serialize>(
    method: Method,
    path: &str,
    body: Option<B>,
    query: Option<&[(&str, &str)]>,
    content_type: &str,
    environment: Environment,
    client: &Client,
    access_token: &str,
) -> Result<Request, APIError> {
    let url = build_url(environment, path, query)?;

    let headers = build_headers(content_type, access_token)?;

    let mut request = client.request(method.clone(), url).headers(headers);

    if method != Method::GET && method != Method::DELETE {
        request = request.json(&body);
    }

    let request = request.build()?;

    log::info!(
        "Built Request with params: {}-{}-{}-{:?}",
        path,
        method,
        if body.is_some() {
            "With JSON Body"
        } else {
            "No JSON Body"
        },
        query
    );

    Ok(request)
}

pub(crate) fn build_url(
    environment: Environment,
    path: &str,
    query: Option<&[(&str, &str)]>,
) -> Result<Url, APIError> {
    let url = Url::parse(environment.endpoint_url())?;
    let mut url = url.join(path)?;
    if let Some(q) = query {
        url.query_pairs_mut()
            .extend_pairs(q.iter())
            .extend_pairs([("minorVersion", "65")]);
    }
    Ok(url)
}
