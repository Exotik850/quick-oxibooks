use quick_oxibooks::actions::{QBCreate, QBQuery};
use quick_oxibooks::error::APIError;
use quick_oxibooks::types::Customer;
use quick_oxibooks::client::Quickbooks;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    let start = std::time::Instant::now();
    let qb = Quickbooks::new_from_env("4620816365257778210", intuit_oxi_auth::Environment::SANDBOX)
        .await?;
    let mut invs = Customer::query(&qb, r#"where GivenName = 'John' and FamilyName = 'Melton'"#)
        .await?;
    let inv = invs.remove(0);
    let new_inv = Customer {
        display_name: Some("John Melton deez".into()),
        family_name: Some("deez".into()),
        id: None,
        sync_token: None,
        ..inv
    };
    new_inv.create(&qb).await?;
    
    let end = start.elapsed();
    println!("Done in {end:?}");
    Ok(())
}
