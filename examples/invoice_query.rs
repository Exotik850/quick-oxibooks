#[cfg(not(feature = "macros"))]
use quick_oxibooks::actions::QBQuery;
#[cfg(feature = "macros")]
use quick_oxibooks::qb_query;
use quick_oxibooks::{client::Quickbooks, error::APIError};
use quickbooks_types::Invoice;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    env_logger::init();

    let mut args = std::env::args().skip(1);

    let token = args.next().expect("Missing Token! 1st Argument");
    let company_id = args.next().expect("Missing Company ID! 2nd Argument");
    let env = args.next().expect("Missing Environment! 3rd Argument");
    let doc_number = args.next().expect("Missing DocNumber! 4th Argument");
    #[cfg(feature = "cache")]
    let key = args.next().expect("Missing Cache Key! 5th Argument");

    let env = match env.as_str() {
        "production" => intuit_oxi_auth::Environment::PRODUCTION,
        "sandbox" => intuit_oxi_auth::Environment::SANDBOX,
        _ => panic!("Invalid environment"),
    };

    let qb = Quickbooks::new_from_token(
        token,
        &company_id,
        env,
        #[cfg(feature = "cache")]
        &key,
    )
    .await?;

    #[cfg(feature = "macros")]
    let inv = qb_query!(&qb, Invoice | doc_number = &doc_number)?;
    #[cfg(not(feature = "macros"))]
    let inv = Invoice::query_single(&qb, &format!("where DocNumber = {}", doc_number)).await?;

    println!("{inv}");

    Ok(())
}
