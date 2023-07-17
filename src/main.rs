mod objects;
mod quickbook;

use crate::{objects::RefreshToken, quickbook::QuickBooks};

fn main() {
    let rt = RefreshToken::get();
    println!("{rt:?}");

    let client_id = dotenv::var("CLIENT_ID").unwrap();
    let client_secret = dotenv::var("CLIENT_SECRET").unwrap();
    let redirect = dotenv::var("REDIRECT_URI").unwrap();

    // let qb = QuickBooks::new(client_id, client_secret, );
}
