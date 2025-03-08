// Currently doesn't support batch voiding,
// not going to be used so will implement when needed

use std::collections::HashMap;

use quickbooks_types::{Invoice, SalesReceipt, Vendor};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    error::{APIError, Fault},
    functions::execute_request,
    QBContext,
};

#[derive(Serialize, Deserialize, Debug)]
struct QBBatchRequest {
    #[serde(rename = "BatchItemRequest")]
    items: Vec<QBBatchItem<QBBatchOperation>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QBResourceOperation {
    #[serde(flatten)]
    pub resource: QBResource,
    pub operation: QBOperationType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum QBOperationType {
    Create,
    Update,
    Delete,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QBBatchItem<T> {
    #[serde(rename = "bId")]
    pub b_id: String,
    #[serde(flatten)]
    pub item: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QBBatchOperation {
    Query(String),
    #[serde(untagged)]
    Operation(QBResourceOperation),
}

impl QBBatchOperation {
    pub fn query(query: impl std::fmt::Display) -> Self {
        QBBatchOperation::Query(query.to_string())
    }

    #[must_use]
    pub fn create(resource: QBResource) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource,
            operation: QBOperationType::Create,
        })
    }

    #[must_use]
    pub fn update(resource: QBResource) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource,
            operation: QBOperationType::Update,
        })
    }

    #[must_use]
    pub fn delete(resource: QBResource) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource,
            operation: QBOperationType::Delete,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QBResource {
    SalesReceipt(SalesReceipt),
    Invoice(Invoice),
    Vendor(Vendor),
    // TODO Add more as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QBQueryResource {
    SalesReceipt(Vec<SalesReceipt>),
    Invoice(Vec<Invoice>),
    // TODO Add more as needed
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QBQueryResult {
    pub start_position: Option<usize>,
    pub max_results: Option<usize>,
    pub total_count: Option<usize>,
    // resource: Vec<T>,
    #[serde(flatten)]
    pub data: Option<QBQueryResource>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QBBatchResponseData {
    Item(QBResource),
    Fault(Fault),
    QueryResponse(QBQueryResult),
}

#[derive(Serialize, Deserialize, Debug)]
struct BatchResponseExt {
    time: String,
    #[serde(rename = "BatchItemResponse")]
    items: Vec<QBBatchItem<QBBatchResponseData>>,
}

pub async fn qb_batch<I>(
    items: I,
    qb: &QBContext,
    client: &reqwest::Client,
    // ) -> Result<Vec<QBBatchItem<QBBatchResponseData>>, APIError>
) -> Result<Vec<(QBBatchOperation, QBBatchResponseData)>, APIError>
where
    I: IntoIterator<Item = QBBatchOperation>,
{
    let batch = QBBatchRequest {
        items: items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let b_id = format!("bId{}", i + 1);
                QBBatchItem { b_id, item }
            })
            .collect(),
    };
    let url = format!("company/{}/batch", qb.company_id);
    let permit = qb
        .batch_limiter
        .acquire()
        .await
        .expect("Semaphore should not be closed");
    let resp = execute_request(qb, client, Method::POST, &url, Some(&batch), None, None).await?;
    // let batch_resp = resp.text().await?;
    let batch_resp: BatchResponseExt = resp.json().await?;
    drop(permit);
    // return Ok(batch_resp);
    let mut items = HashMap::new();

    for item in batch.items {
        items.insert(item.b_id.clone(), item);
    }

    let mut results = Vec::new();
    for resp_item in batch_resp.items {
        if let Some(req_item) = items.remove(&resp_item.b_id) {
            results.push((req_item.item, resp_item.item));
        }
    }

    Ok(results)
}

#[cfg(test)]
mod test {
    use super::{BatchResponseExt, QBBatchRequest};
    #[test]
    fn test_batch_resp() {
        let s = include_str!("../test/data/batch_resp.json");
        let resp: BatchResponseExt = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }

    #[test]
    fn test_batch_req() {
        let s = include_str!("../test/data/batch_req.json");
        let resp: QBBatchRequest = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }
}
