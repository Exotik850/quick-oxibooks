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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
