//! # `QuickBooks` Online API Client
//!
//! This module provides functionality to interact with the `QuickBooks` Online API.
//!
//! ## Usage
//!
//! The primary way to interact with the `QuickBooks` Online API is through the `QBContext` struct,
//! which manages authentication, rate limiting, and API discovery information.
//!
//! ### Creating a Context
//!
//! There are several ways to create a `QBContext`:
//!
//! ```ignore
//! // Create from environment variables
//! let client = ureq::Agent::new_with_defaults();
//! let context = QBContext::new_from_env(Environment::Sandbox, &client)?;
//!
//! // Create manually
//! let context = QBContext::new(
//!     Environment::Production,
//!     "company_id".to_string(),
//!     "access_token".to_string(),
//!     &client
//! )?;
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
//! })?;
//! ```
//!
//! ### Refreshing Tokens
//!
//! If you need to refresh access tokens, use the `RefreshableQBContext`:
//!
//! ```ignore
//! refreshable_context.refresh_access_token("client_id", "client_secret", &client)?;
//! ```
//!
//! ### Rate Limits
//!
//! - Sandbox: 500 requests per minute
//! - Production: 500 requests per minute, 10 requests per second
//! - Batch operations: 30 requests per batch, 40 batches per minute
//!
//! After being throttled, wait 60 seconds before retrying.
use std::time::Duration;

use chrono::{DateTime, Utc};
use ureq::Agent;
// use reqwest::{
//     header::{self, HeaderMap, InvalidHeaderValue},
//     Client, Method, Request,
// };

use super::refresh::RefreshableQBContext;
use crate::{error::QBErrorResponse, limiter::RateLimiter, APIResult, DiscoveryDoc, Environment};

// Rate Limit:
// Sandbox - 500 req / min
/// TODO
// Production - 500 req / min, 10 req / sec
// Batch - 30 req / batch & 40 batches / min
// Wait 60 seconds after throttle

const RATE_LIMIT: usize = 500;
const BATCH_RATE_LIMIT: usize = 40;
const RESET_DURATION: Duration = Duration::from_secs(60);

/// `QuickBooks` Online Context
///
/// This struct holds the context for interacting with the `QuickBooks` Online API.
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
    pub fn new(
        environment: Environment,
        company_id: String,
        access_token: String,
        client: &Agent,
    ) -> APIResult<Self> {
        Ok(Self {
            environment,
            company_id,
            access_token,
            expires_in: Utc::now() + chrono::Duration::hours(999),
            discovery_doc: DiscoveryDoc::get(environment, client)?,
            qbo_limiter: RateLimiter::new(RATE_LIMIT, RESET_DURATION),
            batch_limiter: RateLimiter::new(BATCH_RATE_LIMIT, RESET_DURATION),
        })
    }

    /// Creates a new `QBContext` from environment variables
    ///
    /// Environment variables:
    /// - `QB_COMPANY_ID`
    /// - `QB_ACCESS_TOKEN`
    pub fn new_from_env(environment: Environment, client: &Agent) -> APIResult<Self> {
        let company_id = std::env::var("QB_COMPANY_ID")?;
        let access_token = std::env::var("QB_ACCESS_TOKEN")?;
        let context = Self::new(environment, company_id, access_token, client)?;
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
    pub(crate) fn with_permission<'a, F, T>(&'a self, f: F) -> APIResult<T>
    where
        F: FnOnce(&'a Self) -> APIResult<T>,
    {
        let permit = self.qbo_limiter.acquire();
        let out = f(self);
        drop(permit);
        out
    }

    /// Acquires a permit from the batch rate limiter and executes the given function
    /// with the given context
    pub(crate) fn with_batch_permission<'a, F, T>(&'a self, f: F) -> APIResult<T>
    where
        F: FnOnce(&'a Self) -> APIResult<T>,
    {
        let permit = self.batch_limiter.acquire();
        let out = f(self);
        drop(permit);
        out
    }

    /// Checks if the current context is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_in
    }

    /// Checks if the current access token is authorized
    pub fn check_authorized(&self, client: &Agent) -> APIResult<bool> {
        let request = client
            .get(self.environment.user_info_url())
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .header("Accept", "application/json");
        let response = request.call()?;
        let status = response.status();
        if !status.is_success() {
            log::error!(
                "Failed to check authorized status: {} - {}",
                status,
                response.into_body().read_json::<QBErrorResponse>()?
            );
        }
        Ok(status.is_success())
    }
}
