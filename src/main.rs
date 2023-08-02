use quick_oxibooks::actions::{QBCreate, QBQuery};
use quick_oxibooks::error::APIError;
use quick_oxibooks::types::Customer;
use quick_oxibooks::client::Quickbooks;
use quickbooks_types::{InvoiceBuilder, LineBuilder, LineDetail, SalesItemLineDetail, Item};


#[tokio::main]
async fn main() -> Result<(), APIError> {
    let qb = Quickbooks::new_from_env("4620816365257778210", intuit_oxi_auth::Environment::SANDBOX)
    .await?;
    let start = std::time::Instant::now();
    let cust = Customer::query_single(&qb, r#"where GivenName = 'John' and FamilyName = 'Melton'"#)
    .await?;
    let item = Item::query_single(&qb, "").await?;

    let line = LineBuilder::default()
    .line_detail(Some(LineDetail::SalesItemLineDetail(SalesItemLineDetail{item_ref:item.into(), ..Default::default()})))
    .amount(5.26)
    .build().unwrap();
    // println!("{line}");
    let line = vec![line];

    let new_inv = InvoiceBuilder::default()
    .customer_ref(Some(cust.into()))
    .line(Some(line))
    .build()
    .unwrap();
    // println!("\n{new_inv}");

    new_inv.create(&qb).await?;
    
    let end = start.elapsed();
    println!("Done in {end:?}");
    Ok(())
}
