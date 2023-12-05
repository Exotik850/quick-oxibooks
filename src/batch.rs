use quickbooks_types::{QBItem, *};
use serde::{Deserialize, Serialize};

use crate::{
    client::{Quickbooks, Result}, error::APIError, functions::qb_request
};

// Currently doesn't support batch voiding,
// not going to be used so will implement when needed

#[derive(Debug, Serialize, Deserialize)]
pub enum BatchItem {
    Invoice(Invoice),
    SalesReceipt(SalesReceipt),
    Vendor(Vendor), // Will add more when necessary
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatchItemResp {
    QueryResponse(Vec<BatchItem>),
    #[serde(untagged)]
    Resource(BatchItem),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatchItemRequest {
    #[serde(rename = "BatchItemRequest")]
    pub items: Vec<BatchItemRequestData>,
    #[serde(skip)]
    pub current_id: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatchItemResponse {
    pub items: Vec<BatchItemResponseData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchItemResponseData {
    b_id: String,
    item: BatchItemResp,
}

impl BatchItemRequest {
    pub fn add_command<T: QBItem>(&mut self, operation: BatchOperation, resource: BatchItem) {
        if self.items.len() >= 30 {
            return;
        }
        self.current_id += 1;
        self.items.push(BatchItemRequestData {
            b_id: format!("bid{}", self.current_id),
            data: BatchOption::BatchCommand {
                operation,
                resource,
            },
        });
    }

    pub async fn execute(self, qb: &Quickbooks) -> Result<BatchItemResponse> {
        let response = qb_request!(
            qb,
            reqwest::Method::POST,
            &format!("company/{}/batch", qb.company_id),
            Some(self),
            None
        );
        Ok(response.json().await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchItemRequestData {
    #[serde(rename = "bId")]
    b_id: String,
    #[serde(flatten)]
    data: BatchOption,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatchOption {
    Query(String),
    #[serde(untagged)]
    BatchCommand {
        operation: BatchOperation,
        #[serde(flatten)]
        resource: BatchItem,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchOperation {
    Create,
    Update,
    Delete,
}

#[test]
fn test_batch_load() {
    let val: BatchItemRequest = serde_json::from_str(
        r#"{
    "BatchItemRequest": [
      {
        "bId": "bid1", 
        "Vendor": {
          "DisplayName": "Smith Family Store"
        }, 
        "operation": "create"
      }, 
      {
        "bId": "bid2", 
        "operation": "delete", 
        "Invoice": {
          "SyncToken": "0", 
          "Id": "129"
        }
      }, 
      {
        "SalesReceipt": {
          "PrivateNote": "A private note.", 
          "SyncToken": "0", 
          "domain": "QBO", 
          "Id": "11", 
          "sparse": true
        }, 
        "bId": "bid3", 
        "operation": "update"
      }, 
      {
        "Query": "select * from SalesReceipt where TotalAmt > '300.00'", 
        "bId": "bid4"
      }
    ]
  }"#,
    )
    .unwrap();
    println!("{:?}", val);
}

#[test]
fn test_batch_serialize() {
    let val = BatchItemRequest {
        items: vec![
            BatchItemRequestData {
                b_id: "bId1".into(),
                data: BatchOption::BatchCommand {
                    operation: BatchOperation::Create,
                    resource: serde_json::from_str(r#"{"Invoice": {"a": 1, "c": 2}}"#).unwrap(),
                },
            },
            BatchItemRequestData {
                b_id: "bId2".into(),
                data: BatchOption::BatchCommand {
                    operation: BatchOperation::Delete,
                    resource: serde_json::from_str(r#"{"Invoice": {"a": 1, "c": 2}}"#).unwrap(),
                },
            },
            BatchItemRequestData {
                b_id: "bId3".into(),
                data: BatchOption::Query("* from customer".into()),
            },
            BatchItemRequestData {
                b_id: "bId4".into(),
                data: BatchOption::Query("* from invoice".into()),
            },
        ],
        current_id: 0,
    };

    let value = serde_json::to_string_pretty(&val).unwrap();
    println!("{value}");
}
