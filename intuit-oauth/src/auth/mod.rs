mod token;
pub use token::Environment;

use oauth2::{CsrfToken, ClientId, ClientSecret, RedirectUrl, AuthUrl, TokenUrl, AccessToken, basic::BasicClient, RefreshToken, TokenResponse, Scope, reqwest::async_http_client, AuthorizationCode};
use reqwest::{Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::{net::{TcpStream, TcpListener}, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};

pub const ACCOUNTING_SCOPE: &'static str = "com.intuit.quickbooks.accounting";

#[derive(Debug)]
pub struct APIError {
    pub status_code: StatusCode,
    pub body: String,
}

pub struct Unauthorized {
    client_id: ClientId,
    client_secret: ClientSecret,
}
pub struct Authorized {
    discovery_doc: token::DiscoveryDoc,
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    client: BasicClient
}
pub trait AuthorizeType {}
impl AuthorizeType for Authorized {}
impl AuthorizeType for Unauthorized {}

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
    async fn get_discovery_doc(&self) -> token::DiscoveryDoc {
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

    fn get_consent_url(&self, client: &BasicClient) -> (url::Url, CsrfToken) {
        client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(ACCOUNTING_SCOPE.to_string()))
            .url()
    }

    async fn read_auth_params(stream: &mut TcpStream) -> Option<(AuthorizationCode, CsrfToken)> {

        let mut reader = BufReader::new(stream);
      
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await.unwrap();
      
        let redirect_url = request_line.split_whitespace().nth(1)?;
        let url = url::Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();
      
        let code = url.query_pairs().find(|pair| {
          let &(ref key, _) = pair;
          key == "code"
        })?.1.into_owned();
      
        let state = url.query_pairs().find(|pair| {
          let &(ref key, _) = pair;
          key == "state" 
        })?.1.into_owned();
      
        Some((
          AuthorizationCode::new(code), 
          CsrfToken::new(state)
        ))
      }
    
      async fn handle_oauth_callback(client: &BasicClient, listener: TcpListener, csrf_state: CsrfToken) -> Option<(AccessToken, RefreshToken)> {
        let token_res = loop {
            if let Ok((mut stream, _)) = listener.accept().await {    
                // read redirect response and parse code & state
                let (code, state) = Self::read_auth_params(&mut stream).await.unwrap();
    
                // send 200 OK response
                Self::send_response(&mut stream, "Go back to your terminal :)").await.unwrap();
    
                // check state matches expected
                if state.secret() != csrf_state.secret() {
                    println!("State mismatch!");
                    return None;
                }
    
                // exchange code for token
                let token_res = client.exchange_code(code)
                    .request_async(async_http_client)
                    .await
                    .ok()?;
    
    
                let scopes = if let Some(scopes_vec) = token_res.scopes() {
                    scopes_vec
                        .iter()
                        .map(|comma_separated| comma_separated.split(','))
                        .flatten()
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
    
                break token_res;
            }
        };
        
        // extract access token and refresh token
        let access_token = token_res.access_token().to_owned();
        let refresh_token = token_res.refresh_token().unwrap().to_owned();
    
        Some((access_token, refresh_token))
    }

    async fn send_response(stream: &mut TcpStream, arg: &str) -> Result<(), std::io::Error> {
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            arg.len(),
            arg
        );
        stream.write_all(response.as_bytes()).await
    }
    
}

impl AuthClient<Unauthorized> {

    pub fn new<X, R, Q, O>(client_id: X, client_secret: Q, redirect_uri: R, realm_id: O, environment: Environment) -> Self
    where X: ToString, Q: ToString, R: ToString, O: ToString
    {
        Self {
            redirect_uri: RedirectUrl::new(redirect_uri.to_string()).expect("Invalid redirect URI!"),
            realm_id: realm_id.to_string(),
            environment,
            data: Unauthorized {
                client_id: ClientId::new(client_id.to_string()),
                client_secret: ClientSecret::new(client_secret.to_string()),
            }
        }
    }

    pub fn new_from_env<O>(realm_id: O, environment: Environment) -> Self
    where O: ToString
    {
        let client_id = ClientId::new(dotenv::var("QUICKBOOKS_CLIENT_ID").unwrap());
        let client_secret = ClientSecret::new(dotenv::var("QUICKBOOKS_CLIENT_SECRET").unwrap());
        let redirect_uri = RedirectUrl::new(dotenv::var("QUICKBOOKS_REDIRECT_URI").unwrap()).expect("Failed to parse redirect url");
        Self {
            redirect_uri,
            realm_id: realm_id.to_string(),
            environment,
            data: Unauthorized {
                client_id,
                client_secret, 
            }
        }
    }

    pub async fn authorize(mut self) -> AuthClient<Authorized> {
        let doc = self.get_discovery_doc().await;

        let Unauthorized {client_id, client_secret} = self.data;

        let client = BasicClient::new(client_id,
            Some(client_secret), 
            AuthUrl::new(doc.authorization_endpoint.to_string()).expect("Invalid Auth endpoint from Discovery Doc!"), 
            Some(TokenUrl::new(doc.token_endpoint.to_string()).expect("Invalid Token endpoint from Discovery Doc!"))
        ).set_redirect_uri(self.redirect_uri.clone());

        let (auth_url, csrf_state) = client.authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(ACCOUNTING_SCOPE.to_string()))
            .url();

        println!(
            "Open this URL in your browser:\n{}\n",
            auth_url.to_string()
        );

        let listener = TcpListener::bind("127.0.0.1:3320").await.expect("Error starting localhost callback listener!");
        let (at, rt) = Self::handle_oauth_callback(&client, listener, csrf_state).await.unwrap();

        let data = Authorized {
            discovery_doc: doc,
            access_token: at,
            refresh_token: rt, 
            client,
        };

        AuthClient { 
            realm_id: self.realm_id, 
            redirect_uri: self.redirect_uri, 
            environment: self.environment,
            data
        }
    }
}

impl AuthClient<Authorized> {
    pub async fn refresh_access_token(&mut self) {
        let rtr = self.data.client.exchange_refresh_token(&self.data.refresh_token)
        .request_async(oauth2::reqwest::async_http_client).await.unwrap();
        let at = rtr.access_token().to_owned();
        let rt = rtr.refresh_token().unwrap().to_owned();
        self.data.access_token = at;
        self.data.refresh_token = rt;
    }

    pub fn get_tokens(&self) -> (AccessToken, RefreshToken){
        (self.data.access_token.clone(), self.data.refresh_token.clone())
    }
}
