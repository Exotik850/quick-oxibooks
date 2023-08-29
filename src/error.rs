use quickbooks_types::QBError;
use serde::Serialize;
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] intuit_oxi_auth::AuthError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    TokioIoError(#[from] tokio::io::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    QBError(#[from] QBError),
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
}

impl Serialize for APIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
