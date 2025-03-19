use quick_oxibooks::{attachment::QBUpload, functions::query::QBQuery, QBContext};
use quickbooks_types::{Invoice, QBToRef};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = http_client::h1::H1Client::new();
    let qb = QBContext::new_from_env(quick_oxibooks::Environment::PRODUCTION, &client).await?;

    let invoice = Invoice::query_single("Where DocNumber = '8829'", &qb, &client).await?;

    let attachment = quickbooks_types::Attachable {
        file_name: Some("invoice.pdf".into()),
        note: Some("Invoice attachment".into()),
        attachable_ref: Some(vec![invoice.to_ref()?.into()]),
        ..Default::default()
    };

    let uploaded = attachment.upload(&qb, &client).await?;
    println!("Uploaded: {:?}", uploaded);
    Ok(())
}
