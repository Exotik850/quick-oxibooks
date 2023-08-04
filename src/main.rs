use quick_oxibooks::actions::QBQuery;
use quick_oxibooks::client::Quickbooks;
use quick_oxibooks::error::APIError;
use quickbooks_types::{Invoice, TaxableLine, Preferences};

#[tokio::main]
async fn main() -> Result<(), APIError> {
    let qb = Quickbooks::new_from_env("4620816365257778210", intuit_oxi_auth::Environment::SANDBOX)
        .await?;
    let start = std::time::Instant::now();
    // let cust = Customer::query_single(&qb, r#"where GivenName = 'John' and FamilyName = 'Melton'"#)
    // .await?;
    // let item = Item::query_single(&qb, "").await?;

    // let line = LineBuilder::default()
    // .line_detail(Some(LineDetail::SalesItemLineDetail(SalesItemLineDetail{item_ref:item.into(), ..Default::default()})))
    // .amount(5.26)
    // .build().unwrap();
    // // println!("{line}");
    // let line = vec![line];

    // let new_inv = InvoiceBuilder::default()
    // .customer_ref(Some(cust.into()))
    // .line(Some(line))
    // .build()
    // .unwrap();
    // // println!("\n{new_inv}");

    // new_inv.create(&qb).await?;
    
    let mut inv = Invoice::query_single(&qb, "where DocNumber = '1015'").await?;
    inv.line
        .as_mut()
        .unwrap()
        .iter_mut()
        .for_each(|f| f.set_taxable());
    println!("{inv}");

    let end = start.elapsed();
    println!("Done in {end:?}");
    Ok(())
}
