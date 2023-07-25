use std::error::Error;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct APIError {
    pub status_code: StatusCode,
    pub body: String,
}

impl Error for APIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            status_code: value.status().unwrap_or(StatusCode::EXPECTATION_FAILED),
            body: value.to_string()
        }
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write! (
            f,
            "APIError : Status Code: {} -> {}", 
            self.status_code, self.body
        )
    }
}
