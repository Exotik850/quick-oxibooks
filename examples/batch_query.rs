use quick_oxibooks::{batch::BatchItemRequest, QBContext};

enum ArgFlag {
    AccessToken,
    ObjectType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut transaction_nxrs: Vec<String> = vec![];
    let mut access_token = None;
    let mut object_type = None;
    let mut flag = None;
    for arg in std::env::args().skip(1) {
        // transaction_nxrs.push(arg);
        if flag.is_some() {
            match flag.unwrap() {
                ArgFlag::AccessToken => {
                    access_token = Some(arg);
                }
                ArgFlag::ObjectType => {
                    object_type = Some(arg);
                }
            }
            flag = None;
            continue;
        }
        if arg.trim() == "--access_token" {
            flag = Some(ArgFlag::AccessToken);
            continue;
        }
        if arg.trim() == "--object_type" {
            flag = Some(ArgFlag::ObjectType);
            continue;
        }
        transaction_nxrs.push(arg);
    }
    let object_type = object_type.unwrap_or("SalesReceipt".to_string());

    let client = reqwest::Client::new();
    let mut qb = QBContext::new_from_env(quick_oxibooks::Environment::PRODUCTION, &client).await?;

    if let Some(token) = access_token {
        qb = qb.with_access_token(token)
    }

    let mut batch_items = Vec::new();
    for num in transaction_nxrs {
        batch_items.push(BatchItemRequest::query(dbg!(format!(
            r#"select * from {object_type} where DocNumber = '{num}'"#
        ))));
    }
    let batch_resp = quick_oxibooks::batch::qb_batch(batch_items, &qb, &client).await?;
    for item in batch_resp {
        match item.item {
            quick_oxibooks::batch::BatchItem::QueryResponse(qr) => {
                let msg = qr
                    .data
                    .map(|d| format!("{d:?}"))
                    .unwrap_or_else(|| "None".to_string());
                println!("{}: {}", item.b_id, msg);
            }
            quick_oxibooks::batch::BatchItem::Fault(f) => {
                println!("Error with {}: {:?}, ", item.b_id, f.r#type);
                for fault in f.error {
                    println!(
                        "\t- {} : {}",
                        fault.message,
                        fault.detail.as_deref().unwrap_or("[[No Detail]]")
                    );
                }
            }
            _ => {}
        }
    }

    // println!("{}", batch_resp);
    Ok(())
}
