use serde::{Serialize, Serializer};


#[derive(Debug)]
pub enum APIError {
    ReqwestError(reqwest::Error),
    AuthError(intuit_oauth::AuthError),
    UrlParseError(url::ParseError),
    BadRequest(String),
    NoIdOnRead,
}

impl Serialize for APIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            APIError::ReqwestError(e) => {
                serializer.serialize_str(&format!("ReqwestError: {}", e))
            }
            APIError::AuthError(e) => {
                serializer.serialize_str(&format!("AuthError: {:?}", e)) 
            }
            APIError::UrlParseError(e) => {
                serializer.serialize_str(&format!("UrlParseError: {}", e))
            }
            APIError::BadRequest(msg) => {
                serializer.serialize_str(&format!("BadRequest: {}", msg))
            }
            APIError::NoIdOnRead => {
                serializer.serialize_str("NoIdOnRead")
            }
        }
    }
}

impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<intuit_oauth::AuthError> for APIError {
    fn from(value: intuit_oauth::AuthError) -> Self {
        Self::AuthError(value)
    }
}

impl From<url::ParseError> for APIError {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParseError(value)
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{self:?}"
        )
    }
}
