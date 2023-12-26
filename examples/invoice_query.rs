use quick_oxibooks::{error::APIError, functions::qb_query_single, QBContext};
use quickbooks_types::Invoice;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    env_logger::init();

    let mut args = std::env::args().skip(1);

    let access_token = args.next().expect("Missing Token! 1st Argument");
    let company_id = args.next().expect("Missing Company ID! 2nd Argument");
    let env = args.next().expect("Missing Environment! 3rd Argument");
    let doc_number = args.next().expect("Missing DocNumber! 4th Argument");
    #[cfg(feature = "cache")]
    let key = args.next().expect("Missing Cache Key! 5th Argument");

    let env = match env.as_str() {
        "production" => quick_oxibooks::Environment::PRODUCTION,
        "sandbox" => quick_oxibooks::Environment::SANDBOX,
        _ => panic!("Invalid environment"),
    };

    let qb = QBContext::new(env, company_id, access_token, None);

    let client = reqwest::Client::new();

    let inv: Invoice =
        qb_query_single(&format!(r"where DocNumber = '{doc_number}'"), &qb, &client).await?;

    println!("{inv}");

    Ok(())
}
