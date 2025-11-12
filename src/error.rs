//! Error types for `QuickBooks` API operations.

use quickbooks_types::QBTypeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::batch::{QBBatchOperation, QBBatchResponseData};

/// The main error type for all `QuickBooks` API operations.
///
/// `APIError` is a wrapper around [`APIErrorInner`] that provides a consistent
/// error interface for all operations in the library. It implements standard
/// error traits and can be converted from various underlying error types.
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::error::APIError;
/// use quick_oxibooks::{QBContext, Environment};
/// use ureq::Agent;
///
/// fn create_context() -> Result<QBContext, APIError> {
///     let client = Agent::new_with_defaults();
///     QBContext::new_from_env(Environment::SANDBOX, &client)
/// }
///
/// let _ = match create_context() {
///     Ok(_context) => { println!("Context created successfully"); Ok(()) }
///     Err(e) => { eprintln!("Error: {}", e); Err(e) }
/// };
/// ```
///
/// # Error Conversion
///
/// Many error types automatically convert to `APIError`:
/// - Network errors from `ureq`
/// - JSON parsing errors from `serde_json`
/// - QuickBooks-specific validation errors
/// - Environment variable errors
///
/// # Error Handling Patterns
///
/// ```no_run
/// use quick_oxibooks::error::{APIError, APIErrorInner};
/// use quick_oxibooks::functions::create::QBCreate;
/// use quickbooks_types::{Customer, QBItem};
///
/// fn handle_customer_creation(customer: &Customer, qb_context: &quick_oxibooks::QBContext, client: &ureq::Agent) {
///     match customer.create(qb_context, client) {
///         Ok(created) => println!("Created: {:?}", created.id()),
///         Err(e) => {
///             match &*e {
///                 APIErrorInner::CreateMissingItems => {
///                     eprintln!("Customer missing required fields");
///                 }
///                 APIErrorInner::BadRequest(_qb_error) => {
///                     eprintln!("QuickBooks rejected the request");
///                 }
///                 _ => eprintln!("Other error: {}", e),
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct APIError(Box<APIErrorInner>);

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for APIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl std::ops::Deref for APIError {
    type Target = APIErrorInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for APIError
where
    T: Into<APIErrorInner>,
{
    fn from(err: T) -> Self {
        APIError(Box::new(err.into()))
    }
}

/// Detailed error types for `QuickBooks` API operations.
///
/// This enum contains all the specific error conditions that can occur when
/// interacting with the `QuickBooks` API. Each variant represents a different
/// category of failure with appropriate context information.
///
/// # Error Categories
///
/// ## Network and HTTP Errors
/// - [`UreqError`](APIErrorInner::UreqError): HTTP client errors (network, timeout, etc.)
/// - [`HttpError`](APIErrorInner::HttpError): HTTP protocol errors (malformed requests, etc.)
/// - [`IoError`](APIErrorInner::IoError): I/O errors (file operations, etc.)
///
/// ## API Response Errors
/// - [`BadRequest`](APIErrorInner::BadRequest): `QuickBooks` API returned an error response
/// - [`InvalidClient`](APIErrorInner::InvalidClient): Authentication/authorization failures
/// - [`ThrottleLimitReached`](APIErrorInner::ThrottleLimitReached): Rate limit exceeded
///
/// ## Data Validation Errors
/// - [`QBTypeError`](APIErrorInner::QBTypeError): Entity validation failures
/// - [`CreateMissingItems`](APIErrorInner::CreateMissingItems): Required fields missing for creation
/// - [`DeleteMissingItems`](APIErrorInner::DeleteMissingItems): ID/sync token missing for deletion
/// - [`NoIdOnRead`](APIErrorInner::NoIdOnRead): Entity missing ID for read operation
///
/// ## Query and Operation Errors
/// - [`BatchRequestMissingItems`](APIErrorInner::BatchRequestMissingItems): Batch operation failures
/// - [`BatchLimitExceeded`](APIErrorInner::BatchLimitExceeded): Too many items in batch request
///
/// ## File and Attachment Errors
/// - [`AttachableUploadMissingItems`](APIErrorInner::AttachableUploadMissingItems): Missing required fields for file upload
/// - [`NoAttachableObjects`](APIErrorInner::NoAttachableObjects): No attachments in upload response
/// - [`InvalidFile`](APIErrorInner::InvalidFile): Invalid file name or extension
/// - [`ByteLengthMismatch`](APIErrorInner::ByteLengthMismatch): File write operation incomplete
///
/// ## PDF Generation Errors
/// - [`NoIdOnGetPDF`](APIErrorInner::NoIdOnGetPDF): Entity missing ID for PDF generation
/// - [`NoIdOnSend`](APIErrorInner::NoIdOnSend): Entity missing ID for email send operation
///
/// ## Configuration Errors
/// - [`EnvVarError`](APIErrorInner::EnvVarError): Missing or invalid environment variables
/// - [`JsonError`](APIErrorInner::JsonError): JSON parsing/serialization errors
///
/// # Examples
///
/// ```rust
/// use quick_oxibooks::error::{APIError, APIErrorInner};
/// use quickbooks_types::Customer;
///
/// fn handle_specific_errors(result: Result<Customer, APIError>) {
///     match result {
///         Ok(customer) => println!("Success: {:?}", customer.id),
///         Err(e) => {
///             match &*e {
///                 APIErrorInner::CreateMissingItems => {
///                     eprintln!("Please provide required fields like display_name");
///                 }
///                 APIErrorInner::ThrottleLimitReached => {
///                     eprintln!("Rate limit hit, please wait before retrying");
///                 }
///                 APIErrorInner::BadRequest(qb_error) => {
///                     eprintln!("QuickBooks error: {:?}", qb_error);
///                 }
///                 _ => eprintln!("Other error: {}", e),
///             }
///         }
///     }
/// }
/// ```
// TODO Split this into multiple error types, currently all errors are lumped into one enum
#[derive(Debug, thiserror::Error)]
pub enum APIErrorInner {
    // #[cfg(any(feature = "attachments", feature = "pdf"))]
    // #[error(transparent)]
    // TokioIoError(#[from] tokio::io::Error),
    #[error("Error on Ureq Request: {0}")]
    UreqError(#[from] ureq::Error),
    #[error("HTTP Error: {0}")]
    HttpError(#[from] ureq::http::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Bad request: {0}")]
    BadRequest(QBErrorResponse),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    QBTypeError(#[from] QBTypeError),
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
    #[error("Attachable Missing '{0}' field")]
    AttachableUploadMissingItems(&'static str),
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

// impl From<http_client::http_types::Error> for APIError {
//     fn from(err: http_client::http_types::Error) -> Self {
//         APIErrorInner::HttpError(err).into()
//     }
// }

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum FaultType {
    #[serde(alias = "AUTHENTICATION")]
    Authentication,
    #[serde(rename = "ValidationFault")]
    Validation,
    #[serde(rename = "SystemFault")]
    System,
    // TODO Add the rest of the fault types
    #[serde(untagged)]
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
    pub warnings: Option<Value>,
    pub intuit_object: Option<Value>,
    #[serde(alias = "Fault")]
    pub fault: Option<Fault>,
    pub report: Option<Value>,
    pub sync_error_response: Option<Value>,
    pub query_response: Option<Vec<Value>>,
    pub batch_item_response: Option<Vec<Value>>,
    pub request_id: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "cdcresponse")]
    pub cdc_response: Option<Vec<Value>>,
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
