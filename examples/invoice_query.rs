use quick_oxibooks::{client::Quickbooks, error::APIError, qb_query};
use quickbooks_types::Invoice;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    let mut args = std::env::args().skip(1);

    let token = args.next().expect("Missing Token! 1st Argument");
    let company_id = args.next().expect("Missing Token! 2nd Argument");
    let env = args.next().expect("Missing Environment! 3rd Argument");
    let doc_number = args.next().expect("Missing DocNumber! 4th Argument");
    let key = args.next().expect("Missing Cache Key! 5th Argument");

    let env = match env.as_str() {
        "production" => intuit_oxi_auth::Environment::PRODUCTION,
        "sandbox" => intuit_oxi_auth::Environment::SANDBOX,
        _ => panic!("Invalid environment"),
    };

    let qb = Quickbooks::new_from_token(&token, &company_id, env, &key).await?;

    let inv = qb_query!(&qb, Invoice | doc_number = &doc_number)?;

    println!("{inv}");

    Ok(())
}
