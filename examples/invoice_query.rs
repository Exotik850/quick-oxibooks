use chrono::Utc;
use quick_oxibooks::{error::APIError, functions::qb_query_single, DiscoveryDoc, QBContext};
use quickbooks_types::Invoice;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    env_logger::init();

    let mut args = std::env::args().skip(1);

    let access_token = args.next().expect("Missing Token! 1st Argument");
    let company_id = args.next().expect("Missing Company ID! 2nd Argument");
    let env = args.next().expect("Missing Environment! 3rd Argument");
    let doc_number = args.next().expect("Missing DocNumber! 4th Argument");

    let environment = match env.as_str() {
        "production" => quick_oxibooks::Environment::PRODUCTION,
        "sandbox" => quick_oxibooks::Environment::SANDBOX,
        _ => panic!("Invalid environment"),
    };

    let client = reqwest::Client::new();
    let qb = QBContext::new(environment, company_id, access_token, &client).await?;

    let inv: Invoice =
        qb_query_single(&format!(r"where DocNumber = '{doc_number}'"), &qb, &client).await?;

    println!("{inv}");

    Ok(())
}
