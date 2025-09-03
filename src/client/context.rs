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
//! ```no_run
//! use quick_oxibooks::{QBContext, Environment};
//! use ureq::Agent;
//!
//! // Create from environment variables
//! let client = Agent::new_with_defaults();
//! let context = QBContext::new_from_env(Environment::SANDBOX, &client).unwrap();
//!
//! // Create manually
//! let context = QBContext::new(
//!     Environment::PRODUCTION,
//!     "company_id".to_string(),
//!     "access_token".to_string(),
//!     &client
//! ).unwrap();
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
//! // The context enforces rate limits internally via with_permission/with_batch_permission.
//! // These methods are crate-internal and used by library operations.
//! // End users typically don't call them directly.
//! ```
//!
//! ### Refreshing Tokens
//!
//! If you need to refresh access tokens, use the `RefreshableQBContext`:
//!
//! ```ignore
//! // See RefreshableQBContext docs for usage
//! // refreshable_context.refresh_access_token("client_id", "client_secret", &client)?;
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
use crate::{limiter::RateLimiter, APIResult, DiscoveryDoc, Environment};

// Rate Limit:
// Sandbox - 500 req / min
/// TODO
// Production - 500 req / min, 10 req / sec
// Batch - 30 req / batch & 40 batches / min
// Wait 60 seconds after throttle

const RATE_LIMIT: usize = 500;
const BATCH_RATE_LIMIT: usize = 40;
const RESET_DURATION: Duration = Duration::from_secs(60);

/// The core context for interacting with the `QuickBooks` Online API.
///
/// `QBContext` manages authentication, rate limiting, and API configuration for all
/// `QuickBooks` operations. It automatically handles rate limiting to respect `QuickBooks`
/// API limits and includes discovery document information for OAuth operations.
///
/// # Rate Limits
///
/// - **Regular API**: 500 requests per minute
/// - **Batch API**: 40 batches per minute, 30 requests per batch
/// - **Throttle Recovery**: 60-second wait period after hitting limits
///
/// # Fields
///
/// - `environment`: The `QuickBooks` environment (sandbox or production)
/// - `company_id`: The `QuickBooks` company ID for API requests
/// - `access_token`: OAuth 2.0 access token for authentication
/// - `expires_in`: Token expiration time (defaults to far future)
/// - `discovery_doc`: OAuth discovery document with endpoint URLs
/// - Rate limiters for regular and batch operations
///
/// # Examples
///
/// ## Creating a Context
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
///
/// // Create from explicit parameters
/// let context = QBContext::new(
///     Environment::SANDBOX,
///     "company_123".to_string(),
///     "access_token_xyz".to_string(),
///     &client
/// ).unwrap();
///
/// // Create from environment variables QB_COMPANY_ID and QB_ACCESS_TOKEN
/// let context = QBContext::new_from_env(Environment::SANDBOX, &client).unwrap();
/// ```
///
/// ## Using with Operations
///
/// ```no_run
/// use quick_oxibooks::functions::{create::QBCreate, query::QBQuery};
/// use quickbooks_types::Customer;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let context = quick_oxibooks::QBContext::new(
///     quick_oxibooks::Environment::SANDBOX,
///     "company".to_string(),
///     "token".to_string(),
///     &client
/// ).unwrap();
///
/// // Create a customer
/// let mut customer = Customer::default();
/// customer.display_name = Some("John Doe".to_string());
/// let created = customer.create(&context, &client).unwrap();
///
/// // Query customers
/// let customers = Customer::query("WHERE Active = true", Some(10), &context, &client).unwrap();
/// ```
///
/// ## Refresh Token Support
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let context = QBContext::new(
///     Environment::SANDBOX,
///     "company_123".to_string(),
///     "access_token_xyz".to_string(),
///     &client
/// ).unwrap();
///
/// // Create a refreshable context for automatic token renewal
/// let refreshable = context.with_refresh("refresh_token_abc".to_string());
/// let _ = refreshable;
/// ```
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
    /// Creates a new `QuickBooks` context with the specified parameters.
    ///
    /// This constructor initializes the context with authentication details,
    /// fetches the OAuth discovery document, and sets up rate limiters.
    ///
    /// # Parameters
    ///
    /// - `environment`: `QuickBooks` environment (sandbox or production)
    /// - `company_id`: The `QuickBooks` company ID for your application
    /// - `access_token`: Valid OAuth 2.0 access token
    /// - `client`: HTTP client for making API requests
    ///
    /// # Returns
    ///
    /// Returns a configured `QBContext` on success, or an [`APIError`] on failure.
    ///
    /// # Errors
    ///
    /// - Network errors when fetching the discovery document
    /// - JSON parsing errors if discovery response is malformed
    /// - HTTP errors if discovery endpoint is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quick_oxibooks::{QBContext, Environment};
    /// use ureq::Agent;
    ///
    /// let client = Agent::new_with_defaults();
    /// let context = QBContext::new(
    ///     Environment::SANDBOX,
    ///     "company_123".to_string(),
    ///     "Bearer_token_xyz".to_string(),
    ///     &client
    /// ).unwrap();
    /// ```
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

    /// Creates a new `QuickBooks` context from environment variables.
    ///
    /// This convenience constructor reads the company ID and access token from
    /// environment variables, making it easy to configure the context without
    /// hardcoding credentials.
    ///
    /// # Environment Variables
    ///
    /// - `QB_COMPANY_ID`: The `QuickBooks` company ID
    /// - `QB_ACCESS_TOKEN`: Valid OAuth 2.0 access token
    ///
    /// # Parameters
    ///
    /// - `environment`: `QuickBooks` environment (sandbox or production)
    /// - `client`: HTTP client for making API requests
    ///
    /// # Returns
    ///
    /// Returns a configured `QBContext` on success, or an [`APIError`] on failure.
    ///
    /// # Errors
    ///
    /// - `EnvVarError` if required environment variables are missing
    /// - Network/discovery errors (same as [`QBContext::new`])
    ///
    /// # Examples
    ///
    /// ```bash
    /// export QB_COMPANY_ID="company_123"
    /// export QB_ACCESS_TOKEN="Bearer_token_xyz"
    /// ```
    ///
    /// ```no_run
    /// use quick_oxibooks::{QBContext, Environment};
    /// use ureq::Agent;
    ///
    /// let client = Agent::new_with_defaults();
    /// let context = QBContext::new_from_env(Environment::SANDBOX, &client).unwrap();
    /// ```
    pub fn new_from_env(environment: Environment, client: &Agent) -> APIResult<Self> {
        let company_id = std::env::var("QB_COMPANY_ID")?;
        let access_token = std::env::var("QB_ACCESS_TOKEN")?;
        let context = Self::new(environment, company_id, access_token, client)?;
        Ok(context)
    }

    /// Creates a refreshable context that can automatically renew access tokens.
    ///
    /// Returns a [`RefreshableQBContext`] that wraps this context and provides
    /// automatic token refresh capabilities using the provided refresh token.
    ///
    /// # Parameters
    ///
    /// - `refresh_token`: OAuth 2.0 refresh token for automatic token renewal
    ///
    /// # Returns
    ///
    /// A [`RefreshableQBContext`] that can automatically refresh expired tokens.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quick_oxibooks::{QBContext, Environment};
    /// use ureq::Agent;
    ///
    /// let client = Agent::new_with_defaults();
    /// let context = QBContext::new(
    ///     Environment::SANDBOX,
    ///     "company_123".to_string(),
    ///     "access_token_xyz".to_string(),
    ///     &client
    /// ).unwrap();
    ///
    /// // Enable automatic token refresh
    /// let refreshable = context.with_refresh("refresh_token_abc".to_string());
    /// ```
    #[must_use]
    pub fn with_refresh(self, refresh_token: String) -> RefreshableQBContext {
        RefreshableQBContext {
            context: self,
            refresh_token,
        }
    }

    /// Updates the access token and returns a new context.
    ///
    /// This method is useful when you need to update the access token after
    /// a manual refresh or when switching between different tokens.
    ///
    /// # Parameters
    ///
    /// - `access_token`: The new OAuth 2.0 access token
    ///
    /// # Returns
    ///
    /// A new `QBContext` with the updated access token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quick_oxibooks::{QBContext, Environment};
    /// use ureq::Agent;
    ///
    /// let client = Agent::new_with_defaults();
    /// let context = QBContext::new(
    ///     Environment::SANDBOX,
    ///     "company_123".to_string(),
    ///     "access_token_xyz".to_string(),
    ///     &client
    /// ).unwrap();
    /// // Update the access token after manual refresh
    /// let new_context = context.with_access_token("new_access_token_xyz".to_string());
    /// let _ = new_context; // suppress unused variable warning
    /// ```
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
            #[cfg(feature = "logging")]
            log::error!(
                "Failed to check authorized status: {} - {}",
                status,
                response
                    .into_body()
                    .read_json::<crate::error::QBErrorResponse>()?
            );
        }
        Ok(status.is_success())
    }
}
