// Currently doesn't support batch voiding,
// not going to be used so will implement when needed

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

    pub fn create(resource: QBResource) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource,
            operation: QBOperationType::Create,
        })
    }

    pub fn update(resource: QBResource) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource,
            operation: QBOperationType::Update,
        })
    }

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
) -> Result<Vec<QBBatchItem<QBBatchResponseData>>, APIError>
where
    I: IntoIterator<Item = QBBatchOperation>,
{
    let batch = QBBatchRequest {
        items: items
            .into_iter()
            .enumerate()
            .map(|(i, f)| QBBatchItem {
                b_id: format!("bId{}", i + 1),
                item: f,
            })
            .collect(),
    };
    let url = format!("company/{}/batch", qb.company_id);
    let resp = execute_request(qb, client, Method::POST, &url, Some(batch), None, None).await?;
    // let batch_resp = resp.text().await?;
    let batch_resp: BatchResponseExt = resp.json().await?;
    // return Ok(batch_resp);
    return Ok(batch_resp.items);
}

#[cfg(test)]
mod test {
    use super::{BatchResponseExt, QBBatchRequest};
    #[test]
    fn test_batch_resp() {
        let s = r#"{
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
    }, 
    {
      "bId": "bid4", 
      "QueryResponse": {
        "SalesReceipt": [
          {
            "TxnDate": "2015-08-25", 
            "domain": "QBO", 
            "CurrencyRef": {
              "name": "United States Dollar", 
              "value": "USD"
            }, 
            "PrintStatus": "NotSet", 
            "PaymentRefNum": "10264", 
            "TotalAmt": 337.5, 
            "Line": [
              {
                "Description": "Custom Design", 
                "DetailType": "SalesItemLineDetail", 
                "SalesItemLineDetail": {
                  "TaxCodeRef": {
                    "value": "NON"
                  }, 
                  "Qty": 4.5, 
                  "UnitPrice": 75, 
                  "ItemRef": {
                    "name": "Design", 
                    "value": "4"
                  }
                }, 
                "LineNum": 1, 
                "Amount": 337.5, 
                "Id": "1"
              }, 
              {
                "DetailType": "SubTotalLineDetail", 
                "Amount": 337.5, 
                "SubTotalLineDetail": {}
              }
            ], 
            "ApplyTaxAfterDiscount": false, 
            "DocNumber": "1003", 
            "PrivateNote": "A private note.", 
            "sparse": false, 
            "DepositToAccountRef": {
              "name": "Checking", 
              "value": "35"
            }, 
            "CustomerMemo": {
              "value": "Thank you for your business and have a great day!"
            }, 
            "Balance": 0, 
            "CustomerRef": {
              "name": "Dylan Sollfrank", 
              "value": "6"
            }, 
            "TxnTaxDetail": {
              "TotalTax": 0
            }, 
            "SyncToken": "1", 
            "PaymentMethodRef": {
              "name": "Check", 
              "value": "2"
            }, 
            "EmailStatus": "NotSet", 
            "BillAddr": {
              "Lat": "INVALID", 
              "Long": "INVALID", 
              "Id": "49", 
              "Line1": "Dylan Sollfrank"
            }, 
            "MetaData": {
              "CreateTime": "2015-08-27T14:59:48-07:00", 
              "LastUpdatedTime": "2016-04-15T09:01:10-07:00"
            }, 
            "CustomField": [
              {
                "DefinitionId": "1", 
                "Type": "StringType", 
                "Name": "Crew #"
              }
            ], 
            "Id": "11"
          }
        ], 
        "startPosition": 1, 
        "maxResults": 1
      }
    }
  ], 
  "time": "2016-04-15T09:01:18.141-07:00"
}"#;
        let resp: BatchResponseExt = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }

    #[test]
    fn test_batch_req() {
        let s = r#"{
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
}"#;
        let resp: QBBatchRequest = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }
}
