use oauth2::{RequestTokenError, StandardErrorResponse, basic::BasicErrorResponseType, reqwest::Error};

#[derive(Debug)]
pub enum AuthError {
    UnsuccessfulRequest,
    ReqwestError(reqwest::Error),
    ParseError(url::ParseError),
    EnvVarError(std::env::VarError),
    IoError(std::io::Error),
    // Very ugly, might do a PR on oauth2 to clean up
    RequestTokenError(RequestTokenError<Error<reqwest::Error>, StandardErrorResponse<BasicErrorResponseType>>),
    StateMismatch,
    NoRedirectUrl,
    KeyNotFound(&'static str),
    NoTokenResponse,
}

impl From<reqwest::Error> for AuthError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<url::ParseError> for AuthError {
    fn from(value: url::ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::env::VarError> for AuthError {
    fn from(value: std::env::VarError) -> Self {
        Self::EnvVarError(value)
    }
}

impl From<std::io::Error> for AuthError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<RequestTokenError<Error<reqwest::Error>, StandardErrorResponse<BasicErrorResponseType>>> for AuthError {
    fn from(value: RequestTokenError<Error<reqwest::Error>, StandardErrorResponse<BasicErrorResponseType>>) -> Self {
        Self::RequestTokenError(value)
    }
}