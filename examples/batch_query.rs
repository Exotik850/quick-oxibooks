use quick_oxibooks::{batch::{BatchIterator, QBBatchOperation}, QBContext};

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
        qb = qb.with_access_token(token);
    }

    let mut batch_items = Vec::new();
    for num in transaction_nxrs {
        batch_items.push(QBBatchOperation::query(dbg!(format!(
            r#"select * from {object_type} where DocNumber = '{num}'"#
        ))));
    }
    let batch_resp = batch_items.batch(&qb, &client).await?;
    for (op, item) in batch_resp {
        match item {
            quick_oxibooks::batch::QBBatchResponseData::QueryResponse(qr) => {
                let msg = qr.data.map_or_else(|| "None", |_| "Found");
                println!("{op:?}: {msg}");
            }
            quick_oxibooks::batch::QBBatchResponseData::Fault(f) => {
                println!("Error with {:?}: {:?}, ", op, f.r#type);
                for fault in f.error {
                    println!(
                        "\t- {} : {}",
                        fault.message,
                        fault.detail.as_deref().unwrap_or("[[No Detail]]")
                    );
                }
            }
            _ => {
                println!("{op:?}: {item:?}");
            }
        }
    }

    // println!("{}", batch_resp);
    Ok(())
}
