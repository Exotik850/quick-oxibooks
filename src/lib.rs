/*
 * A rust library for interacting with the QuickBooks API.
 *
 * For more information, you can check out their documentation at:
 * https://developer.intuit.com/app/developer/qbo/docs/develop
 *
 * ORIGINIALLY FROM https://github.com/oxidecomputer/cio
 * LICENSED UNDER APACHE 2.0
 *
 */
#![warn(clippy::pedantic)]

pub mod batch;
pub mod client;
pub use client::QBContext;
use error::APIError;
use serde::{Deserialize, Serialize};
pub mod error;

pub mod types {
    pub use quickbooks_types::*;
}

pub mod functions;

#[cfg(feature = "attachments")]
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
    pub async fn get_async(environment: Environment) -> Result<Self, APIError> {
        let url = environment.discovery_url();
        let resp = reqwest::get(url).await?;
        if !resp.status().is_success() {
            return Err(APIError::BadTokenRequest(resp.text().await?));
        }
        let out: Self = resp.json().await?;
        Ok(out)
    }

    pub fn get(environment: Environment) -> Result<Self, APIError> {
        let url = environment.discovery_url();
        let resp = reqwest::blocking::get(url)?;
        if !resp.status().is_success() {
            return Err(APIError::BadTokenRequest(resp.text()?));
        }
        let out: Self = resp.json()?;
        Ok(out)
    }
}
