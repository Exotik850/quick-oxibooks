use std::sync::Arc;

use intuit_oxi_auth::{Environment, AuthClient};
use reqwest::Client;

use crate::error::APIError;


pub type Result<T> = std::result::Result<T, APIError>;

/// Entrypoint for interacting with the QuickBooks API.
#[derive(Debug, Clone)]
pub struct Quickbooks<T>
{
    pub(crate) company_id: String,
    pub environment: Environment,
    pub(crate) client: Arc<AuthClient<T>>,
    pub(crate) http_client: Arc<Client>,
}

pub trait QBData<T> {
    fn get_data(&self) -> Option<&T>;
}