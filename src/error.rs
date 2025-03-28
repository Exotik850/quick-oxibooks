use quickbooks_types::QBTypeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::batch::{QBBatchOperation, QBBatchResponseData};
// #[allow(dead_code)]
// TODO Split this into multiple error types, currently all errors are lumped into one enum
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    // #[cfg(any(feature = "attachments", feature = "pdf"))]
    // #[error(transparent)]
    // TokioIoError(#[from] tokio::io::Error),
    #[error("Error on HTTP Request: {0}")]
    HttpError(http_client::http_types::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Bad request: {0}")]
    BadRequest(QBErrorResponse),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    QBTypeError(#[from] QBTypeError),
    #[error("No query objects returned for query_str : {0}")]
    NoQueryObjects(String),
    #[error("Invalid Client! Try re-authenticating")]
    InvalidClient,
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
    #[error("Invalid File name or extenstion : {0}")]
    InvalidFile(String),
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

impl From<http_client::http_types::Error> for APIError {
    fn from(err: http_client::http_types::Error) -> Self {
        APIError::HttpError(err)
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
pub enum FaultType {
    #[serde(alias = "AUTHENTICATION")]
    Authentication,
    #[serde(rename = "ValidationFault")]
    Validation,
    #[serde(rename = "SystemFault")]
    System,
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
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QBErrorResponse {
    pub(crate) warnings: Option<Value>,
    pub(crate) intuit_object: Option<Value>,
    #[serde(alias = "Fault")]
    pub(crate) fault: Option<Fault>,
    pub(crate) report: Option<Value>,
    pub(crate) sync_error_response: Option<Value>,
    pub(crate) query_response: Option<Vec<Value>>,
    pub(crate) batch_item_response: Option<Vec<Value>>,
    pub(crate) request_id: Option<String>,
    pub(crate) time: String,
    pub(crate) status: Option<String>,
    #[serde(rename = "cdcresponse")]
    pub(crate) cdc_response: Option<Vec<Value>>,
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
