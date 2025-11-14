//! # Quick Oxibooks Library
//!
//! **Quick Oxibooks** is a comprehensive Rust library for interacting with the `QuickBooks` Online API.
//! It provides a strongly-typed, rate-limited, and feature-rich interface for performing CRUD operations,
//! batch processing, and generating reports with `QuickBooks` data.
//!
//! ## Key Features
//!
//! - **Complete CRUD Operations**: Create, read, update, delete, and query `QuickBooks` entities
//! - **Batch Processing**: Execute multiple operations in a single API call for improved performance
//! - **Rate Limiting**: Built-in rate limiting to respect `QuickBooks` API limits
//! - **Strong Typing**: All `QuickBooks` entities are strongly typed with validation
//! - **PDF Generation**: Generate PDFs for supported entities (invoices, estimates, etc.)
//! - **Attachment Support**: Upload and manage file attachments
//! - **Report Generation**: Access `QuickBooks` financial reports (P&L, Balance Sheet, etc.)
//! - **Macro Support**: Convenient macros for building queries
//!
//! ## Quick Start
//!
//! ```no_run
//! use quick_oxibooks::{QBContext, Environment};
//! use quickbooks_types::{Customer, Invoice};
//! use quick_oxibooks::functions::{create::QBCreate, query::QBQuery, read::QBRead};
//! use ureq::Agent;
//!
//! // Create a QuickBooks context
//! let client = Agent::new_with_defaults();
//! let qb_context = QBContext::new(
//!     Environment::SANDBOX,
//!     "your_company_id".to_string(),
//!     "your_access_token".to_string(),
//!     &client
//! ).unwrap();
//!
//! // Create a new customer
//! let mut customer = Customer::default();
//! customer.display_name = Some("John Doe".to_string());
//! let created_customer = customer.create(&qb_context, &client).unwrap();
//!
//! // Query for invoices
//! let invoices = Invoice::query("WHERE TotalAmt > '100.00'", Some(10), &qb_context, &client).unwrap();
//!
//! // Read a specific invoice by ID
//! let invoice = Invoice::query_single("WHERE Id = '123'", &qb_context, &client).unwrap();
//! ```
//! ## Features
//!
//! - `attachments`: Enables file attachment upload and management functions
//! - `pdf`: Enables PDF generation for supported `QuickBooks` entities
//! - `macros`: Enables convenient query-building macros
//! - `polars`: Enables integration with the `polars` `DataFrame` library for data analysis
//! - `logging`: Enables detailed logging of API requests and responses
//!
//! For more detailed usage examples, refer to the documentation for each module and type.
#![warn(clippy::pedantic)]

pub mod batch;
pub mod client;
pub use client::QBContext;
use error::APIError;
use serde::{Deserialize, Serialize};
use ureq::Agent;
pub mod error;

pub mod types {
    //! Re-exports of all types from the `quickbooks_types` crate.
    pub use quickbooks_types::*;
}

pub mod functions;
pub(crate) mod limiter;

use crate::error::APIErrorInner;
#[cfg(feature = "attachments")]
pub use crate::functions::attachment;
#[cfg(feature = "pdf")]
pub use crate::functions::pdf;
#[cfg(feature = "macros")]
pub mod macros;

/// The result type used throughout the library for operations that may fail.
///
/// This is a type alias for `Result<T, APIError>` where `APIError` contains
/// detailed information about what went wrong during API operations.
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::{APIResult, QBContext, Environment};
/// use ureq::Agent;
///
/// fn get_context() -> APIResult<QBContext> {
///     let client = Agent::new_with_defaults();
///     QBContext::new_from_env(Environment::SANDBOX, &client)
/// }
/// ```
pub type APIResult<T> = Result<T, APIError>;

/// Represents the `QuickBooks` API environment.
///
/// `QuickBooks` provides two environments:
/// - **SANDBOX**: For development and testing, uses sandbox URLs and data
/// - **PRODUCTION**: For live applications, uses production URLs and real data
///
/// The environment determines which API endpoints are used for all operations.
///
/// # Examples
///
/// ```rust
/// use quick_oxibooks::Environment;
///
/// // For development
/// let env = Environment::SANDBOX;
///
/// // For production
/// let env = Environment::PRODUCTION;
///
/// // Default is SANDBOX for safety
/// let default_env = Environment::default();
/// assert_eq!(default_env, Environment::SANDBOX);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Environment {
    /// Production environment for live `QuickBooks` data
    PRODUCTION,
    /// Sandbox environment for development and testing (default)
    #[default]
    SANDBOX,
}

impl Environment {
    /// Returns the OAuth 2.0 discovery URL for the environment.
    ///
    /// The discovery URL provides OAuth endpoints and configuration for authentication.
    ///
    /// # Returns
    ///
    /// A static string containing the discovery URL for the environment.
    #[inline]
    #[must_use]
    pub fn discovery_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => {
                "https://developer.intuit.com/.well-known/openid_configuration/"
            }
            Environment::SANDBOX => {
                "https://developer.intuit.com/.well-known/openid_sandbox_configuration/"
            }
        }
    }

    /// Returns the token migration URL for the environment.
    ///
    /// Used for migrating OAuth 1.0 tokens to OAuth 2.0.
    ///
    /// # Returns
    ///
    /// A static string containing the migration URL for the environment.
    #[inline]
    #[must_use]
    pub fn migration_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => {
                "https://developer-sandbox.api.intuit.com/v2/oauth2/tokens/migrate"
            }
            Environment::SANDBOX => "https://developer.api.intuit.com/v2/oauth2/tokens/migrate",
        }
    }

    /// Returns the user info URL for the environment.
    ///
    /// Used to retrieve user information from the `OpenID` Connect userinfo endpoint.
    ///
    /// # Returns
    ///
    /// A static string containing the user info URL for the environment.
    #[inline]
    #[must_use]
    pub fn user_info_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => {
                "https://accounts.platform.intuit.com/v1/openid_connect/userinfo"
            }
            Environment::SANDBOX => {
                "https://sandbox-accounts.platform.intuit.com/v1/openid_connect/userinfo"
            }
        }
    }

    /// Returns the base API endpoint URL for the environment.
    ///
    /// This is the root URL for all `QuickBooks` API operations (CRUD, queries, reports, etc.).
    ///
    /// # Returns
    ///
    /// A static string containing the API endpoint URL for the environment.
    #[inline]
    #[must_use]
    pub fn endpoint_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://quickbooks.api.intuit.com/v3/",
            Environment::SANDBOX => "https://sandbox-quickbooks.api.intuit.com/v3/",
        }
    }
}

/// OAuth 2.0 discovery document for `QuickBooks` API.
///
/// Contains OAuth 2.0 endpoint URLs and supported capabilities discovered from
/// the `QuickBooks` OAuth discovery endpoint. This is automatically fetched when
/// creating a [`QBContext`] and used for authentication flows.
///
/// # Fields
///
/// - `issuer`: The OAuth 2.0 issuer identifier
/// - `authorization_endpoint`: URL for user authorization
/// - `token_endpoint`: URL for token exchange
/// - `userinfo_endpoint`: URL for retrieving user information
/// - `revocation_endpoint`: URL for token revocation
/// - `jwks_uri`: URL for JSON Web Key Set
/// - Various supported capabilities arrays
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::{DiscoveryDoc, Environment};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let discovery = DiscoveryDoc::get(Environment::SANDBOX, &client).unwrap();
/// println!("Token endpoint: {}", discovery.token_endpoint);
/// ```
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(unused)]
pub struct DiscoveryDoc {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub revocation_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

impl DiscoveryDoc {
    /// Fetches the OAuth 2.0 discovery document from `QuickBooks`.
    ///
    /// This method makes an HTTP request to the `QuickBooks` discovery endpoint
    /// to retrieve OAuth 2.0 configuration and supported capabilities.
    ///
    /// # Parameters
    ///
    /// - `environment`: The `QuickBooks` environment (sandbox or production)
    /// - `client`: HTTP client for making the request
    ///
    /// # Returns
    ///
    /// Returns the parsed discovery document on success, or an [`APIError`] on failure.
    ///
    /// # Errors
    ///
    /// - Network errors when fetching the discovery document
    /// - JSON parsing errors if the response format is invalid
    /// - HTTP errors if the discovery endpoint returns an error response
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quick_oxibooks::{DiscoveryDoc, Environment};
    /// use ureq::Agent;
    ///
    /// let client = Agent::new_with_defaults();
    /// let discovery = DiscoveryDoc::get(Environment::SANDBOX, &client).unwrap();
    /// ```
    pub fn get(environment: Environment, client: &Agent) -> APIResult<Self> {
        let url = environment.discovery_url();
        let request = client.get(url).call()?;
        if !request.status().is_success() {
            return Err(APIErrorInner::BadRequest(request.into_body().read_json()?).into());
        }
        Ok(request.into_body().read_json()?)
    }
}
