use quick_oxibooks::{error::APIError, functions::query::QBQuery, QBContext};
use quickbooks_types::Invoice;

// #[tokio::main]
fn main() -> Result<(), APIError> {
    env_logger::init();

    let mut args = std::env::args().skip(1);

    let access_token = args.next().expect("Missing Token! 1st Argument");
    let company_id = args.next().expect("Missing Company ID! 2nd Argument");
    let env = args.next().expect("Missing Environment! 3rd Argument");
    let doc_number = args.next().expect("Missing DocNumber! 4th Argument");

    let environment = match env.to_lowercase().as_str() {
        "production" => quick_oxibooks::Environment::PRODUCTION,
        "sandbox" => quick_oxibooks::Environment::SANDBOX,
        _ => panic!("Invalid environment"),
    };

    let client = ureq::Agent::new_with_defaults();
    let qb = QBContext::new(environment, company_id, access_token, &client)?;

    let inv = Invoice::query_single(&format!(r"where DocNumber = '{doc_number}'"), &qb, &client)?;

    println!("{inv:?}");

    Ok(())
}
