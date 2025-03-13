//! # QuickBooks Online API Client
//!
//! This module provides functionality to interact with the QuickBooks Online API.
//!
//! ## Usage
//!
//! The primary way to interact with the QuickBooks Online API is through the `QBContext` struct,
//! which manages authentication, rate limiting, and API discovery information.
//!
//! ### Creating a Context
//!
//! There are several ways to create a `QBContext`:
//!
//! ```ignore
//! // Create from environment variables
//! let client = http_client::h1::H1Client::new();
//! let context = QBContext::new_from_env(Environment::Sandbox, &client).await?;
//!
//! // Create manually
//! let context = QBContext::new(
//!     Environment::Production,
//!     "company_id".to_string(),
//!     "access_token".to_string(),
//!     &client
//! ).await?;
//!
//! // Create with refresh token capability
//! let refreshable_context = context.with_refresh("refresh_token".to_string());
//! ```
//!
//! ### Handling Rate Limits
//!
//! The context automatically handles rate limiting for both regular API calls and batch operations.
//! When making API calls, use the `with_permission` or `with_batch_permission` methods to respect rate limits:
//!
//! ```ignore
//! context.with_permission(|ctx| async {
//!     // Your API call here that uses ctx
//! }).await?;
//! ```
//!
//! ### Refreshing Tokens
//!
//! If you need to refresh access tokens, use the `RefreshableQBContext`:
//!
//! ```
//! refreshable_context.refresh_access_token("client_id", "client_secret", &client).await?;
//! ```
//!
//! ### Rate Limits
//!
//! - Sandbox: 500 requests per minute
//! - Production: 500 requests per minute, 10 requests per second
//! - Batch operations: 30 requests per batch, 40 batches per minute
//!
//! After being throttled, wait 60 seconds before retrying.
use std::{future::Future, ops::Deref, time::Duration};

use base64::Engine;
use chrono::{DateTime, Utc};
use http_client::{http_types::Method, HttpClient, Request};
// use reqwest::{
//     header::{self, HeaderMap, InvalidHeaderValue},
//     Client, Method, Request,
// };
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    error::{APIError, QBErrorResponse},
    limiter::RateLimiter,
    DiscoveryDoc, Environment,
};

use super::refresh::RefreshableQBContext;

// Rate Limit:
// Sandbox - 500 req / min
/// TODO
// Production - 500 req / min, 10 req / sec
// Batch - 30 req / batch & 40 batches / min
// Wait 60 seconds after throttle

const RATE_LIMIT: usize = 500;
const BATCH_RATE_LIMIT: usize = 40;
const RESET_DURATION: Duration = Duration::from_secs(60);

/// QuickBooks Online Context
///
/// This struct holds the context for interacting with the QuickBooks Online API.
/// It includes authentication details, rate limiters, and discovery document.
///
/// Note: The `expires_in` field is set to a far future date by default and should be updated upon token refresh.
pub struct QBContext {
    pub(crate) environment: Environment,
    pub(crate) company_id: String,
    pub(crate) access_token: String,
    pub(crate) expires_in: DateTime<Utc>,
    pub(crate) discovery_doc: DiscoveryDoc,
    pub(crate) qbo_limiter: RateLimiter,
    pub(crate) batch_limiter: RateLimiter, // Batch endpoints have a different rate limit
}

impl QBContext {
    /// Creates a new `QBContext` with the given parameters
    pub async fn new<Client: HttpClient>(
        environment: Environment,
        company_id: String,
        access_token: String,
        client: &Client,
    ) -> Result<Self, APIError> {
        Ok(Self {
            environment,
            company_id,
            access_token,
            expires_in: Utc::now() + chrono::Duration::hours(999),
            discovery_doc: DiscoveryDoc::get_async(environment, client).await?,
            qbo_limiter: RateLimiter::new(RATE_LIMIT, RESET_DURATION),
            batch_limiter: RateLimiter::new(BATCH_RATE_LIMIT, RESET_DURATION),
        })
    }

    /// Creates a new `QBContext` from environment variables
    ///
    /// Environment variables:
    /// - `QB_COMPANY_ID`
    /// - `QB_ACCESS_TOKEN`
    pub async fn new_from_env<Client: HttpClient>(
        environment: Environment,
        client: &Client,
    ) -> Result<Self, APIError> {
        let company_id = std::env::var("QB_COMPANY_ID")?;
        let access_token = std::env::var("QB_ACCESS_TOKEN")?;
        let context = Self::new(environment, company_id, access_token, client).await?;
        Ok(context)
    }

    /// Creates a `RefreshableQBContext` from the current context and a refresh token
    #[must_use]
    pub fn with_refresh(self, refresh_token: String) -> RefreshableQBContext {
        RefreshableQBContext {
            context: self,
            refresh_token,
        }
    }

    /// Updates the access token in the context
    #[must_use]
    pub fn with_access_token(self, access_token: String) -> Self {
        Self {
            access_token,
            ..self
        }
    }

    /// Acquires a permit from the rate limiter and executes the given function
    /// with the given context
    pub(crate) async fn with_permission<'a, F, FF, T>(&'a self, f: F) -> Result<T, APIError>
    where
        F: FnOnce(&'a Self) -> FF,
        FF: Future<Output = Result<T, APIError>>,
    {
        let permit = self
            .qbo_limiter
            .acquire()
            .await
            .expect("Semaphore should not be closed");
        let out = f(self).await;
        drop(permit);
        out
    }

    /// Acquires a permit from the batch rate limiter and executes the given function
    /// with the given context
    pub(crate) async fn with_batch_permission<'a, F, FF, T>(&'a self, f: F) -> Result<T, APIError>
    where
        F: FnOnce(&'a Self) -> FF,
        FF: Future<Output = Result<T, APIError>>,
    {
        let permit = self
            .batch_limiter
            .acquire()
            .await
            .expect("Semaphore should not be closed");
        let out = f(self).await;
        drop(permit);
        out
    }

    /// Checks if the current context is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_in
    }

    /// Checks if the current access token is authorized
    pub async fn check_authorized<Client: HttpClient>(
        &self,
        client: &Client,
    ) -> Result<bool, APIError> {
        let mut request = Request::new(Method::Get, self.environment.user_info_url());
        request.insert_header("Authorization", format!("Bearer {}", &self.access_token));
        request.insert_header("Accept", "application/json");
        let mut response = client.send(request).await?;
        let status = response.status();
        if !status.is_success() {
            log::error!(
                "Failed to check authorized status: {} - {}",
                status,
                response.body_json::<QBErrorResponse>().await?
            );
        }
        Ok(status.is_success())
    }
}
