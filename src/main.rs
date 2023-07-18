mod objects;
mod quickbook;
mod auth;
use std::{net::TcpListener, io::{BufReader, BufRead, Write}};

use auth::{AUTH_ENDPOINT, TOKEN_ENDPOINT, ACCOUNTING_SCOPE};
// use auth::{AuthClient, Environment};
use oauth2::{basic::BasicClient, ClientId, ClientSecret, RedirectUrl, AuthUrl, TokenUrl, CsrfToken, Scope, AuthorizationCode, reqwest::http_client, StandardRevocableToken, TokenResponse, PkceCodeChallenge};
use reqwest::Url;
// use crate::{objects::RefreshToken, quickbook::QuickBooks};


#[tokio::main]
async fn main() {

    let client_id = ClientId::new(dotenv::var("QUICKBOOKS_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(dotenv::var("QUICKBOOKS_CLIENT_SECRET").unwrap());
    let redirect_uri = RedirectUrl::new(dotenv::var("QUICKBOOKS_REDIRECT_URI").unwrap()).expect("Failed to parse redirect url");

    let auth_url = AuthUrl::new(AUTH_ENDPOINT.to_owned())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(TOKEN_ENDPOINT.to_owned())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));
    // .set_redirect_uri(redirect_uri);

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
    .authorize_url(CsrfToken::new_random)
    // This example is requesting access to the "calendar" features and the user's profile.
    .add_scope(Scope::new(
        ACCOUNTING_SCOPE.to_string(),
    ))
    .set_pkce_challenge(pkce_code_challenge)
    .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:3320").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            println!("Google returned the following code:\n{}\n", code.secret());
            println!(
                "Google returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(http_client);

            println!(
                "Google returned the following token:\n{:?}\n",
                token_response
            );

            // Revoke the obtained token
            let token_response = token_response.unwrap();
            let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
                Some(token) => token.into(),
                None => token_response.access_token().into(),
            };

            client
                .revoke_token(token_to_revoke)
                .unwrap()
                .request(http_client)
                .expect("Failed to revoke token");

            // The server will terminate itself after revoking the token.
            break;
        }
    }


    

    // let qb = QuickBooks::new_from_env(4620816365257778210i64, auth_token, refresh);

    // let inv = qb.company_info("4620816365257778210").await.unwrap();
    // println!("{inv:?}");
}
