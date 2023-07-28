use oauth2::{RequestTokenError, StandardErrorResponse, basic::BasicErrorResponseType, reqwest::Error};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Request unsuccessful")]
    UnsuccessfulRequest,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
    #[error(transparent)]
    EnvVarError(#[from] std::env::VarError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    // Very ugly, might do a PR on oauth2 to clean up
    #[error(transparent)]
    RequestTokenError(#[from] RequestTokenError<Error<reqwest::Error>, StandardErrorResponse<BasicErrorResponseType>>),
    #[error("State mismatch on authentication")]
    StateMismatch,
    #[error("Redirect URL not found, try putting it in your environment variables with the name INTUIT_REDIRECT_URI")]
    NoRedirectUrl,
    #[error("Key not found in authentication response: {0}")]
    KeyNotFound(&'static str),
    #[error("No response when trying to authorize")]
    NoTokenResponse,
}