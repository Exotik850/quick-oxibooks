//! # Quick Oxibooks Library
//!
//! This library provides a Rust interface for interacting with the `QuickBooks` API.
//! It includes modules for handling various aspects of the API, such as batch operations,
//! client management, error handling, and more.
//!
//! ## Modules
//!
//! - `batch`: Contains functionality for batch operations.
//! - `client`: Manages the `QuickBooks` client context.
//! - `error`: Defines error types used throughout the library.
//! - `types`: Re-exports types from the `quickbooks_types` crate.
//! - `functions`: Contains various utility functions for interacting with the API.
//! - `limiter`: Provides rate limiting functionality (crate-private).
//! - `macros`: Contains macros for use with the library (optional).
//!
//! ## Features
//!
//! - `attachments`: Enables attachment-related functions.
//! - `pdf`: Enables PDF-related functions.
//! - `macros`: Enables macros for use with the library.
//!
//! ## Enums
//!
//! - `Environment`: Represents the environment (production or sandbox) for the `QuickBooks` API.
//!
//! ## Structs
//!
//! - `DiscoveryDoc`: Represents the discovery document for the `QuickBooks` API.
//!
//! ## Usage
//!
//! To use this library, add it as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! quick-oxibooks = "0.1.0"
//! ```
//!
//! Then, import the necessary modules and types in your code:
//!
//! ```rust
//! use quick_oxibooks::client::QBContext;
//! use quick_oxibooks::Environment;
//! ```
//!
//! For more detailed usage examples, refer to the documentation for each module and type.
#![warn(clippy::pedantic)]

pub mod batch;
pub mod client;
pub use client::QBContext;
use error::APIError;
use http_client::{HttpClient, Request};
use serde::{Deserialize, Serialize};
pub mod error;

pub mod types {
    pub use quickbooks_types::*;
}

pub mod functions;
pub(crate) mod limiter;

pub use crate::functions::attachment;
#[cfg(feature = "pdf")]
pub use crate::functions::pdf;

#[cfg(feature = "macros")]
pub mod macros;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Environment {
    PRODUCTION,
    #[default]
    SANDBOX,
}

impl Environment {
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

    #[inline]
    #[must_use]
    pub fn endpoint_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://quickbooks.api.intuit.com/v3/",
            Environment::SANDBOX => "https://sandbox-quickbooks.api.intuit.com/v3/",
        }
    }
}

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
    pub async fn get_async<Client: HttpClient>(
        environment: Environment,
        client: &Client,
    ) -> Result<Self, APIError> {
        let url = environment.discovery_url();
        // let request = client.get(url).build()?;
        let request = Request::get(url);
        let mut resp = client.send(request).await?;
        if !resp.status().is_success() {
            return Err(APIError::BadRequest(resp.body_json().await?));
        }
        Ok(resp.body_json().await?)
    }

    // pub fn get(
    //     environment: Environment,
    //     client: &reqwest::blocking::Client,
    // ) -> Result<Self, APIError> {
    //     let url = environment.discovery_url();
    //     let request = client.get(url).build()?;
    //     let resp = client.execute(request)?;
    //     if !resp.status().is_success() {
    //         return Err(APIError::BadRequest(resp.json()?));
    //     }
    //     Ok(resp.json()?)
    // }
}
