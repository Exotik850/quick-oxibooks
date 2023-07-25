mod error;
mod functions;
mod quickbook;
use functions::query::QBQuery;
use quickbook::Quickbooks;
use quickbooks_types::models::Invoice;

#[tokio::main]
async fn main() {
    let qb =
        Quickbooks::new_from_env("4620816365257778210", intuit_oauth::Environment::SANDBOX).await;
    let invs = Invoice::query(&qb, r#""#).await.unwrap();

    for inv in invs.iter() {
        println!("{inv}");
    }
}
