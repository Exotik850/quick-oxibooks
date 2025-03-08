use quickbooks_types::QBTypeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::batch::{QBBatchOperation, QBBatchResponseData};
// #[allow(dead_code)]
// TODO Split this into multiple error types, currently all errors are lumped into one enum
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[cfg(any(feature = "attachments", feature = "pdf"))]
    #[error(transparent)]
    TokioIoError(#[from] tokio::io::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Bad request: {0}")]
    BadRequest(QBErrorResponse),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    QBTypeError(#[from] QBTypeError),
    #[error("No query objects returned for query_str : {0}")]
    NoQueryObjects(String),
    #[error("Trying to update an object when it doesn't have an ID set")]
    NoIdOnRead,
    #[error("Trying to send object email when it doesn't have an ID set")]
    NoIdOnSend,
    #[error("Missing objects when trying to create item")]
    CreateMissingItems,
    #[error("Can't delete objects without ID or SyncToken")]
    DeleteMissingItems,
    #[error("Missing ID when trying to get PDF of object")]
    NoIdOnGetPDF,
    #[error("Couldn't write all the bytes of file")]
    ByteLengthMismatch,
    #[error("Missing either Note or Filename when uploading Attachable")]
    AttachableUploadMissingItems,
    #[error("Missing Attachable object on upload response")]
    NoAttachableObjects,
    #[error("Throttle limit reached")]
    ThrottleLimitReached,
    #[error("Batch limit exceeded")]
    BatchLimitExceeded,
    #[error("Env Var error : {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("Invalid Batch Response, Missing items for : {0}")]
    BatchRequestMissingItems(BatchMissingItemsError),
}

#[derive(Debug, thiserror::Error)]
pub struct BatchMissingItemsError {
    pub items: std::collections::HashMap<String, crate::batch::QBBatchOperation>,
    pub results: Vec<(QBBatchOperation, QBBatchResponseData)>,
}

impl std::fmt::Display for BatchMissingItemsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BatchRequestMissingItems : {{ Missing Items: {:#?}, \n Results : {:#?} }}",
            self.items, self.results
        )
    }
}

impl Serialize for APIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QBError {
    #[serde(alias = "Message")]
    pub message: String,
    pub code: String,
    #[serde(alias = "Detail")]
    pub detail: Option<String>,
    pub element: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "UPPERCASE")]
pub enum FaultType {
    Authentication,
    #[serde(rename = "ValidationFault")]
    Validation,
    // TODO Add the rest of the fault types
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fault {
    pub r#type: FaultType,
    #[serde(alias = "Error")]
    pub error: Vec<QBError>,
}

// TODO Make the fields more strongly typed, currently no documentation on the error types that I can find
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QBErrorResponse {
    warnings: Option<Value>,
    intuit_object: Option<Value>,
    fault: Option<Fault>,
    report: Option<Value>,
    sync_error_response: Option<Value>,
    query_response: Option<Vec<Value>>,
    batch_item_response: Vec<Value>,
    request_id: Option<String>,
    time: u64,
    status: Option<String>,
    #[serde(rename = "cdcresponse")]
    cdc_response: Vec<Value>,
}

impl std::fmt::Display for QBErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self)
                .expect("Could not serialize QBErrorResponse for display!")
        )
    }
}
