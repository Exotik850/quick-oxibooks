
#[derive(Debug)]
pub enum APIError {
    ReqwestError(reqwest::Error),
    AuthError(),
    NoIdOnRead,
    
}

impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
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
