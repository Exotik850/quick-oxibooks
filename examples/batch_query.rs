use quick_oxibooks::{batch::BatchItemRequest, QBContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transaction_nxrs: Vec<String> = std::env::args().skip(1).collect();
    let client = reqwest::Client::new();
    let qb = QBContext::new_from_env(quick_oxibooks::Environment::PRODUCTION, &client).await?;

    let mut batch_items = Vec::new();
    for num in transaction_nxrs {
        batch_items.push(BatchItemRequest::query(format!(
            r#"select * from Invoice where DocNumber = '{num}'"#
        )));
    }
    let batch_resp = quick_oxibooks::batch::qb_batch(batch_items, &qb, &client).await?;
    for item in batch_resp {
        match item.item {
            quick_oxibooks::batch::BatchItem::QueryResponse(qr) => {
                println!("{}: {:?}", item.b_id, qr.data);
            }
            quick_oxibooks::batch::BatchItem::Fault(f) => {
                println!("{}: {:?}", item.b_id, f.r#type);
            }
            _ => {}
        }
    }

    Ok(())
}
