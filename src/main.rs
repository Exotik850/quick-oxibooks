
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl, AccessToken, RefreshToken,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
mod quickbook;
mod objects;
use quickbook::{QuickBooks};

pub const AUTH_ENDPOINT: &'static str = "https://appcenter.intuit.com/connect/oauth2";
pub const TOKEN_ENDPOINT: &'static str =
    "https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer";
pub const REVOKE_ENDPOINT: &'static str =
    "https://developer.api.intuit.com/v2/oauth2/tokens/revoke";
pub const ACCOUNTING_SCOPE: &'static str = "com.intuit.quickbooks.accounting";

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
            let (code, state) = read_auth_params(&mut stream).await.unwrap();

            // send 200 OK response
            send_response(&mut stream, "Go back to your terminal :)").await.unwrap();

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
            println!("Returned the following scopes:\n{:?}\n", scopes);

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


#[tokio::main]
async fn main() {
    let client_id = ClientId::new(dotenv::var("QUICKBOOKS_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(dotenv::var("QUICKBOOKS_CLIENT_SECRET").unwrap());
    // let redirect_uri = RedirectUrl::new(dotenv::var("QUICKBOOKS_REDIRECT_URI").unwrap()).expect("Failed to parse redirect url");
    let redirect_uri =
        RedirectUrl::new("http://localhost:3320".to_owned()).expect("Couldn't parse redirect url");

    let auth_url =
        AuthUrl::new(AUTH_ENDPOINT.to_owned()).expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(TOKEN_ENDPOINT.to_owned()).expect("Invalid token endpoint URL");

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_uri);

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(ACCOUNTING_SCOPE.to_string()))
        .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    // Start a server to listen on the redirect URI
    let listener = TcpListener::bind("127.0.0.1:3320").await.unwrap();
    let (at, rt) = handle_oauth_callback(&client, listener, csrf_state).await.unwrap();

    // println!("{:?}: {:?}", at.secret(), rt.secret());

    let qb = QuickBooks::new_from_env("4620816365257778210", at.secret(), rt.secret());
    
    let ci = qb.company_info("4620816365257778210").await.unwrap();
    println!("{ci:?}");
}
