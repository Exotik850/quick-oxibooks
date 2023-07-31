mod error;
mod functions;
mod client;
use functions::query::QBQuery;
use client::Quickbooks;
use quickbooks_types::models::Customer;

#[tokio::main]
async fn main() {
    let qb = Quickbooks::new_from_env("4620816365257778210", intuit_oxi_auth::Environment::SANDBOX)
        .await
        .unwrap();
    let invs = Customer::query(&qb, r#"where GivenName = 'John' and FamilyName = 'Melton'"#)
        .await
        .unwrap();
    println!("{}", invs[0])
}
