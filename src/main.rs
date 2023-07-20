
use std::io::{Write, Read};
mod quickbook;
mod objects;
use quickbook::QuickBooks;
use intuit_oauth::{AuthClient, oauth2::{AccessToken, RefreshToken}};

#[tokio::main]
async fn main() {
    let (at, rt): (AccessToken, RefreshToken);
    if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open("refresh.txt") {
        at = AccessToken::new("".to_owned());
        let mut read = String::new();
        file.read_to_string(&mut read).unwrap();
        rt = RefreshToken::new(read);
    } else {
        let auth = AuthClient::new_from_env("4620816365257778210", intuit_oauth::Environment::SANDBOX).await;
        let mut auth = auth.authorize().await;
        auth.refresh_access_token().await;
        (at, rt) = auth.get_tokens();
    }
    let mut qb = QuickBooks::new_from_env("4620816365257778210", at.secret(), rt.secret());
    qb.refresh_access_token().await.unwrap();
    
    let ci = qb.company_info("4620816365257778210").await.unwrap();
    println!("{ci:?}");

    let inv = qb.get_invoice_by_doc_num("1010").await.unwrap();
    println!("{inv:?}");

    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).create(true).open("refresh.txt") {
        write!(file, "{}", qb.refresh_token).unwrap();
        println!("Saved refresh token")
    }
}
