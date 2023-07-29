mod error;
mod functions;
mod quickbook;
use functions::query::QBQuery;
use quickbook::Quickbooks;
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
