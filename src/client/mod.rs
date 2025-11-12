//! # `QuickBooks` Online API Client
//!
//! This module provides functionality to interact with the `QuickBooks` Online API.
//!
//! ## Usage
//!
//! The primary way to interact with the `QuickBooks` Online API is through the [`QBContext`] struct,
//! which manages authentication, rate limiting, and API discovery information.
//!
//! ### Creating a Context
//!
//! There are several ways to create a [`QBContext`]:
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
//! ### Handling Refresh Tokens
//!
//! The [`RefreshableQBContext`] struct extends [`QBContext`] to support automatic token refreshing.
//!
//! This is useful for long-running applications that need to maintain access to the `QuickBooks` Online API,
//! or use on a desktop application where the user may not be able to go through the OAuth flow frequently.
//!
//! ### Rate Limits
//!
//! - Sandbox: 500 requests per minute
//! - Production: 500 requests per minute, 10 requests per second
//! - Batch operations: 30 requests per batch, 40 batches per minute
//!
//! After being throttled, wait 60 seconds before retrying.

use crate::{APIResult, Environment};
use serde::Serialize;
use ureq::{
    http::{request::Builder, Method, Request},
    SendBody,
};
mod context;
mod refresh;
pub use context::QBContext;
pub use refresh::RefreshableQBContext;
use urlencoding::encode;

pub(crate) fn set_headers(content_type: &str, access_token: &str, request: Builder) -> Builder {
    let bt = format!("Bearer {access_token}");
    let mut request = request.header("Authorization", bt);
    if content_type != "multipart/form-data" {
        request = request.header("Content-Type", content_type);
    }
    request.header("Accept", "application/json")
}

pub(crate) fn build_request<B, S, SS>(
    method: &Method,
    path: &str,
    body: Option<&B>,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
    content_type: &str,
    environment: Environment,
    access_token: &str,
) -> APIResult<Request<SendBody<'static>>>
where
    B: Serialize,
    S: AsRef<str>,
    SS: AsRef<str>,
{
    let url = build_url(environment, path, query);
    let mut request = Request::builder().method(method.clone()).uri(url.as_str());
    request = set_headers(content_type, access_token, request);

    let request = match (method == Method::GET || method == Method::DELETE, body) {
        (true, _) | (false, None) => request.body(SendBody::none()),
        (false, Some(body)) => {
            let json_bytes = serde_json::to_vec(body)?;
            let reader = std::io::Cursor::new(json_bytes);
            request.body(SendBody::from_owned_reader(reader))
        }
    }?;

    #[cfg(feature = "logging")]
    log::debug!(
        "Built Request with params: {}-{}-{}",
        path,
        method,
        if body.is_some() {
            "With JSON Body"
        } else {
            "No JSON Body"
        },
    );

    Ok(request)
}

pub(crate) fn build_url<S, SS>(
    environment: Environment,
    path: &str,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
) -> String
where
    S: AsRef<str>,
    SS: AsRef<str>,
{
    // let url = Url::parse(environment.endpoint_url())?;
    let mut url = environment.endpoint_url().to_string();
    url.push_str(path);
    if let Some(q) = query {
        let query_string: String = q
            .into_iter()
            .map(|(k, v)| {
                (
                    encode(k.as_ref()).to_string(),
                    encode(v.as_ref()).to_string(),
                )
            })
            .map(|(k, v)| format!("{k}={v}"))
            .chain(std::iter::once("minorversion=75".to_string()))
            .collect::<Vec<_>>()
            .join("&");
        if !query_string.is_empty() {
            url.push('?');
            url.push_str(&query_string);
        }
    }
    url
}
