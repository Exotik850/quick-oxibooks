use quick_oxibooks::{attachment::QBUpload, functions::query::QBQuery, QBContext};
use quickbooks_types::{Invoice, QBToRef};

enum ArgFlag {
    AccessToken,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut access_token = None;
    let mut flag = None;
    for arg in std::env::args().skip(1) {
        // transaction_nxrs.push(arg);
        if flag.is_some() {
            match flag.unwrap() {
                ArgFlag::AccessToken => {
                    access_token = Some(arg);
                }
            }
            flag = None;
            continue;
        }
        if arg.trim() == "--access_token" {
            flag = Some(ArgFlag::AccessToken);
            continue;
        }
    }
    // let client = reqwest::Client::new();
    let client = http_client::h1::H1Client::new();
    let mut qb = QBContext::new_from_env(quick_oxibooks::Environment::PRODUCTION, &client).await?;

    if let Some(token) = access_token {
        println!("Found access token");
        qb = qb.with_access_token(token);
    }

    let invoice = Invoice::query_single("Where DocNumber = '8827W'", &qb, &client).await?;

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
