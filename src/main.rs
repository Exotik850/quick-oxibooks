mod error;
mod functions;
mod quickbook;
use functions::query::QBQuery;
use quickbook::Quickbooks;
use quickbooks_types::models::Invoice;

#[tokio::main]
async fn main() {
    let qb =
        Quickbooks::new_from_env("4620816365257778210", intuit_oauth::Environment::SANDBOX).await.unwrap();
    let invs = Invoice::query(&qb, r#"where DocNumber = '1010'"#).await.unwrap();
    println!("{}", invs[0])
}
