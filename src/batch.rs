// Currently doesn't support batch voiding,
// not going to be used so will implement when needed

use chrono::Utc;
use quickbooks_types::{Invoice, QBItem, SalesReceipt, Vendor};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{client::QBContext, error::APIError, functions::qb_request, DiscoveryDoc};

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchItemData<T> {
    #[serde(rename = "bId")]
    b_id: String,
    #[serde(flatten)]
    item: T,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatchItemRequest {
    #[serde(rename = "BatchItemRequest")]
    pub items: Vec<BatchItemData<BatchOption>>,
    #[serde(skip)]
    pub current_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatchOption {
    Query(String),
    // #[serde(flatten)]
    #[serde(untagged)]
    BatchCommand(BatchCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCommand {
    operation: BatchOperation,
    #[serde(flatten)]
    resource: BatchItem,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatchItem {
    Invoice(Invoice),
    SalesReceipt(SalesReceipt),
    Vendor(Vendor), // TODO Add more types
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatchItemResponse {
    #[serde(rename = "BatchItemResponse")]
    pub items: Vec<BatchItemData<BatchItemResp>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BatchItemResp {
    QueryResponse(Vec<BatchItem>),
    Fault(BatchFault),
    #[serde(untagged)]
    Resource(BatchItem),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFault {
    r#type: String,
    #[serde(rename = "Error")]
    error: Vec<BatchFaultError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFaultError {
    #[serde(rename = "Message")]
    message: String,
    code: String,
    #[serde(rename = "Detail")]
    detail: String,
    element: String,
}

impl BatchItemRequest {
    pub fn add_command(&mut self, item: BatchOption) -> Option<usize> {
        if self.items.len() >= 30 {
            return None;
        }
        self.current_id += 1;
        self.items.push(BatchItemData {
            b_id: format!("bid{}", self.current_id),
            item,
        });
        Some(self.current_id)
    }

    pub async fn execute(
        self,
        qb: &QBContext,
        client: &Client,
    ) -> Result<BatchItemResponse, APIError> {
        if self.items.is_empty() {
            return Ok(BatchItemResponse { items: vec![] });
        }

        qb_request(
            qb,
            client,
            reqwest::Method::POST,
            &format!("company/{}/batch", qb.company_id),
            Some(self),
            None,
            None,
        )
        .await
    }
}
impl std::ops::Index<usize> for BatchItemRequest {
    type Output = BatchItemData<BatchOption>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn batch_req_load() {
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
        println!("{val:?}");
    }

    #[test]
    fn batch_req_serialize() {
        let val = BatchItemRequest {
            items: vec![
                BatchItemData {
                    b_id: "bId1".into(),
                    item: BatchOption::BatchCommand(BatchCommand {
                        operation: BatchOperation::Create,
                        resource: serde_json::from_str(r#"{"Invoice": {"a": 1, "c": 2}}"#).unwrap(),
                    }),
                },
                BatchItemData {
                    b_id: "bId2".into(),
                    item: BatchOption::BatchCommand(BatchCommand {
                        operation: BatchOperation::Delete,
                        resource: serde_json::from_str(r#"{"Invoice": {"a": 1, "c": 2}}"#).unwrap(),
                    }),
                },
                BatchItemData {
                    b_id: "bId3".into(),
                    item: BatchOption::Query("* from customer".into()),
                },
                BatchItemData {
                    b_id: "bId4".into(),
                    item: BatchOption::Query("* from invoice".into()),
                },
            ],
            current_id: 0,
        };

        let value = serde_json::to_string_pretty(&val).unwrap();
        println!("{value}");
    }

    #[test]
    fn batch_resp_load() {
        let val = r#"{
    "BatchItemResponse": [
      {
        "Fault": {
          "type": "ValidationFault", 
          "Error": [
            {
              "Message": "Duplicate Name Exists Error", 
              "code": "6240", 
              "Detail": "The name supplied already exists. : Another customer, vendor or employee is already using this \nname. Please use a different name.", 
              "element": ""
            }
          ]
        }, 
        "bId": "bid1"
      }, 
      {
        "Fault": {
          "type": "ValidationFault", 
          "Error": [
            {
              "Message": "Object Not Found", 
              "code": "610", 
              "Detail": "Object Not Found : Something you're trying to use has been made inactive. Check the fields with accounts, customers, items, vendors or employees.", 
              "element": ""
            }
          ]
        }, 
        "bId": "bid2"
      }, 
      {
        "Fault": {
          "type": "ValidationFault", 
          "Error": [
            {
              "Message": "Stale Object Error", 
              "code": "5010", 
              "Detail": "Stale Object Error : You and root were working on this at the same time. root finished before you did, so your work was not saved.", 
              "element": ""
            }
          ]
        }, 
        "bId": "bid3"
      }

    ], 
    "time": "2016-04-15T09:01:18.141-07:00"
  }"#;

        let resp: BatchItemResponse = serde_json::from_str(val).unwrap();
        println!("{resp:?}");
    }
}
// {
//   "bId": "bid4",
//   "QueryResponse": {
//     "SalesReceipt": [
//       {
//         "TxnDate": "2015-08-25",
//         "domain": "QBO",
//         "CurrencyRef": {
//           "name": "United States Dollar",
//           "value": "USD"
//         },
//         "PrintStatus": "NotSet",
//         "PaymentRefNum": "10264",
//         "TotalAmt": 337.5,
//         "Line": [
//           {
//             "Description": "Custom Design",
//             "DetailType": "SalesItemLineDetail",
//             "SalesItemLineDetail": {
//               "TaxCodeRef": {
//                 "value": "NON"
//               },
//               "Qty": 4.5,
//               "UnitPrice": 75,
//               "ItemRef": {
//                 "name": "Design",
//                 "value": "4"
//               }
//             },
//             "LineNum": 1,
//             "Amount": 337.5,
//             "Id": "1"
//           },
//           {
//             "DetailType": "SubTotalLineDetail",
//             "Amount": 337.5,
//             "SubTotalLineDetail": {}
//           }
//         ],
//         "ApplyTaxAfterDiscount": false,
//         "DocNumber": "1003",
//         "PrivateNote": "A private note.",
//         "sparse": false,
//         "DepositToAccountRef": {
//           "name": "Checking",
//           "value": "35"
//         },
//         "CustomerMemo": {
//           "value": "Thank you for your business and have a great day!"
//         },
//         "Balance": 0,
//         "CustomerRef": {
//           "name": "Dylan Sollfrank",
//           "value": "6"
//         },
//         "TxnTaxDetail": {
//           "TotalTax": 0
//         },
//         "SyncToken": "1",
//         "PaymentMethodRef": {
//           "name": "Check",
//           "value": "2"
//         },
//         "EmailStatus": "NotSet",
//         "BillAddr": {
//           "Lat": "INVALID",
//           "Long": "INVALID",
//           "Id": "49",
//           "Line1": "Dylan Sollfrank"
//         },
//         "MetaData": {
//           "CreateTime": "2015-08-27T14:59:48-07:00",
//           "LastUpdatedTime": "2016-04-15T09:01:10-07:00"
//         },
//         "CustomField": [
//           {
//             "DefinitionId": "1",
//             "Type": "StringType",
//             "Name": "Crew #"
//           }
//         ],
//         "Id": "11"
//       }
//     ],
//     "startPosition": 1,
//     "maxResults": 1
//   }
// }
