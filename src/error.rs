
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