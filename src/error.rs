use serde::Serialize;
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] intuit_oauth::AuthError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Trying to update an object when it doesn't have an ID set")]
    NoIdOnRead,
}

impl Serialize for APIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
