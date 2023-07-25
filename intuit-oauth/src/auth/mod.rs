mod token;
mod error;
pub use token::Environment;
pub use error::AuthError;

use oauth2::{CsrfToken, ClientId, ClientSecret, RedirectUrl, AuthUrl, TokenUrl, AccessToken, basic::BasicClient, RefreshToken, TokenResponse, Scope, reqwest::async_http_client, AuthorizationCode};
use tokio::{net::{TcpStream, TcpListener}, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};
use url::Url;

use self::token::DiscoveryDoc;

pub const ACCOUNTING_SCOPE: &'static str = "com.intuit.quickbooks.accounting";

pub struct Unauthorized {
    client_id: ClientId,
    client_secret: ClientSecret,
    discovery_doc: DiscoveryDoc,
}
pub struct Authorized {
    access_token: AccessToken,
    refresh_token: RefreshToken,
    client: BasicClient
}
pub trait AuthorizeType {}
impl AuthorizeType for Authorized {}
impl AuthorizeType for Unauthorized {}

#[derive(Debug)]
pub struct AuthClient<T>
where T: AuthorizeType
{
    realm_id: String,
    redirect_uri: RedirectUrl,
    environment: Environment,
    pub data: T
}

impl<T> AuthClient<T> 
where T: AuthorizeType
{
    async fn get_discovery_doc(environment: &Environment) -> Result<DiscoveryDoc, AuthError> {
        let url = environment.discovery_url();
        let resp = reqwest::get(url).await?;
        if !resp.status().is_success() {
            return Err(AuthError::UnsuccessfulRequest)
        }
        let out: DiscoveryDoc = resp.json().await?;
        Ok(out)
    }

    async fn read_auth_params(stream: &mut TcpStream) -> Result<(AuthorizationCode, CsrfToken), AuthError> {

        let mut reader = BufReader::new(stream);
      
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await.unwrap();
      
        let redirect_url = match request_line.split_whitespace().nth(1) {
            Some(it) => it,
            None => return Err(AuthError::NoRedirectUrl),
        };
        let url = url::Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();
      
        let code = match url.query_pairs().find(|pair| {
                  let &(ref key, _) = pair;
                  key == "code"
                }) {
            Some(it) => it,
            None => return Err(AuthError::KeyNotFound("code")),
        }.1.into_owned();
      
        let state = match url.query_pairs().find(|pair| {
                  let &(ref key, _) = pair;
                  key == "state" 
                }) {
            Some(it) => it,
            None => return Err(AuthError::KeyNotFound("state")),
        }.1.into_owned();
      
        Ok((
          AuthorizationCode::new(code), 
          CsrfToken::new(state)
        ))
      }
    
      async fn handle_oauth_callback(client: &BasicClient, listener: TcpListener, csrf_state: CsrfToken) -> Result<(AccessToken, RefreshToken), AuthError> {
        let token_res = loop {
            if let Ok((mut stream, _)) = listener.accept().await {    
                let (code, state) = Self::read_auth_params(&mut stream).await?;
    
                Self::send_ok_response(&mut stream, "Go back to your terminal :)").await.unwrap();
    
                if state.secret() != csrf_state.secret() {
                    return Err(AuthError::StateMismatch);
                }
    
                let token_res = match client.exchange_code(code)
                                    .request_async(async_http_client)
                                    .await
                                    .ok() {
                    Some(it) => it,
                    None => return Err(AuthError::NoTokenResponse),
                };
    
                break token_res;
            }
        };
        
        // extract access token and refresh token
        let access_token = token_res.access_token().to_owned();
        let refresh_token = token_res.refresh_token().unwrap().to_owned();
    
        Ok((access_token, refresh_token))
    }

    async fn send_ok_response(stream: &mut TcpStream, arg: &str) -> Result<(), std::io::Error> {
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            arg.len(),
            arg
        );
        stream.write_all(response.as_bytes()).await
    }    
}

impl AuthClient<Unauthorized> {

    pub async fn new<X, R, Q, O>(client_id: X, client_secret: Q, redirect_uri: R, realm_id: O, environment: Environment) -> Result<Self, AuthError>
    where X: ToString, Q: ToString, R: ToString, O: ToString
    {
        let discovery_doc = Self::get_discovery_doc(&environment).await?;

        Ok(Self {
            redirect_uri: RedirectUrl::new(redirect_uri.to_string())?,
            realm_id: realm_id.to_string(),
            environment,
            data: Unauthorized {
                client_id: ClientId::new(client_id.to_string()),
                client_secret: ClientSecret::new(client_secret.to_string()),
                discovery_doc
            }
        })
    }

    pub async fn new_from_env<O>(realm_id: O, environment: Environment) -> Result<Self, AuthError>
    where O: ToString
    {
        let discovery_doc = Self::get_discovery_doc(&environment).await?;

        let client_id = ClientId::new(dotenv::var("INTUIT_CLIENT_ID")?);
        let client_secret = ClientSecret::new(dotenv::var("INTUIT_CLIENT_SECRET")?);
        let redirect_uri = RedirectUrl::new(dotenv::var("INTUIT_REDIRECT_URI")?)?;
        Ok(Self {
            redirect_uri,
            realm_id: realm_id.to_string(),
            environment,
            data: Unauthorized {
                client_id,
                client_secret, 
                discovery_doc
            }
        })
    }

    pub async fn authorize(self) -> Result<AuthClient<Authorized>, AuthError> {

        let Unauthorized {client_id, client_secret, discovery_doc} = self.data;

        let auth_url = AuthUrl::new(discovery_doc.authorization_endpoint.to_string())?;
        let token_url = Some(TokenUrl::new(discovery_doc.token_endpoint.to_string())?);

        let client = BasicClient::new(client_id,Some(client_secret), auth_url, token_url)
        .set_redirect_uri(self.redirect_uri.clone());

        let (auth_url, csrf_state) = client.authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(ACCOUNTING_SCOPE.to_string()))
        .url();

        match open::that(auth_url.as_str()) {
            Ok(_) => println!("Opened auth url successfully!\n"),
            Err(e) => return Err(e.into()),
        }

        let (at, rt): (AccessToken, RefreshToken);

        match self.environment {
            Environment::PRODUCTION => unimplemented!(),
            Environment::SANDBOX => {
                let listener = TcpListener::bind("127.0.0.1:3320").await.expect("Error starting localhost callback listener!");
                (at, rt) = Self::handle_oauth_callback(&client, listener, csrf_state).await.unwrap();        
            },
        }

        let data = Authorized {
            access_token: at,
            refresh_token: rt, 
            client,
        };

        Ok(AuthClient { 
            realm_id: self.realm_id, 
            redirect_uri: self.redirect_uri, 
            environment: self.environment,
            data
        })
    }
}

impl AuthClient<Authorized> {
    pub async fn refresh_access_token(&mut self) -> Result<(), AuthError> {
        let rtr = match self.data.client.exchange_refresh_token(&self.data.refresh_token)
                .request_async(oauth2::reqwest::async_http_client).await {
            Ok(it) => it,
            Err(err) => return Err(err.into()),
        };
        let at = rtr.access_token().to_owned();
        let rt = rtr.refresh_token().unwrap().to_owned();
        self.data.access_token = at;
        self.data.refresh_token = rt;
        Ok(())
    }

    pub fn get_tokens(&self) -> (AccessToken, RefreshToken){
        (self.data.access_token.clone(), self.data.refresh_token.clone())
    }

    pub fn get_auth_url(&self) -> (Url, CsrfToken) {
        self.data.client.authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(ACCOUNTING_SCOPE.to_string()))
        .url()
    }
}
