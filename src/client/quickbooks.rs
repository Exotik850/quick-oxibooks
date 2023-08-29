use std::sync::Arc;

use intuit_oxi_auth::{AuthClient, Authorized, Environment};
use reqwest::Client;

use crate::error::APIError;

pub type Result<T> = std::result::Result<T, APIError>;

/// Entrypoint for interacting with the `QuickBooks` API.
#[derive(Debug)]
pub struct Quickbooks {
    pub(crate) company_id: String,
    pub environment: Environment,
    pub(crate) client: AuthClient<Authorized>,
    pub(crate) http_client: Arc<Client>,
    #[cfg(feature = "cache")]
    pub(crate) key: String,
}
