use std::sync::Arc;

use intuit_oxi_auth::{AuthClient, Environment};
use reqwest::Client;

use crate::error::APIError;

pub type Result<T> = std::result::Result<T, APIError>;

/// Entrypoint for interacting with the QuickBooks API.
#[derive(Debug, Clone)]
pub struct Quickbooks<T> {
    pub(crate) company_id: String,
    pub environment: Environment,
    pub(crate) client: Arc<AuthClient<T>>,
    pub(crate) http_client: Arc<Client>,
}

#[cfg(feature = "cache")]
use intuit_oxi_auth::Cache;

#[cfg(feature = "cache")]
impl<T: Cache> Quickbooks<T>
{
    pub fn cleanup(&self) {
        let file_name = self.environment.cache_name();
        self.client.cleanup(file_name)
    }
}