//! Module for `QuickBooks` API functions
//!
//! This module contains functions for interacting with the `QuickBooks` API, including
//! creating, reading, updating, and deleting various `QuickBooks` entities.

use quickbooks_types::{QBItem, QBSendable};
use serde::{Deserialize, Serialize};
use ureq::http::Response;
use ureq::Body;
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    APIResult, QBContext,
};

#[cfg(feature = "attachments")]
pub mod attachment;
pub mod create;
pub mod delete;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod query;
pub mod read;
pub mod reports;

/// Sends a request to the `QuickBooks` API endpoint with the given parameters,
/// accounts for rate limiting
///
/// # Arguments
///
/// * `qb` - The context containing authentication details
/// * `method` - The HTTP method for the request
/// * `path` - The path for the API request URL
/// * `body` - Optional request body to send
/// * `content_type` - Optional content type header value
/// * `query` - Optional query parameters
pub(crate) fn qb_request<S, SS, T, U>(
    qb: &QBContext,
    client: &Agent,
    method: Method,
    path: &str,
    body: Option<&T>,
    content_type: Option<&str>,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
) -> APIResult<U>
where
    T: Serialize,
    U: serde::de::DeserializeOwned,
    S: AsRef<str>,
    SS: AsRef<str>,
{
    let response = qb.with_permission(|qb| {
        execute_request(qb, client, method, path, body, content_type, query)
    })?;
    Ok(response.into_body().read_json()?)
}

pub(crate) fn execute_request<S, SS, T: Serialize>(
    qb: &QBContext,
    client: &Agent,
    method: Method,
    path: &str,
    body: Option<&T>,
    content_type: Option<&str>,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
) -> Result<Response<Body>, APIError>
where
    S: AsRef<str>,
    SS: AsRef<str>,
{
    let request = crate::client::build_request(
        &method,
        path,
        body,
        query,
        content_type.unwrap_or("application/json"),
        qb.environment,
        &qb.access_token,
    )?;
    let response = client.run(request)?;
    if !response.status().is_success() {
        return Err(APIErrorInner::BadRequest(response.into_body().read_json()?).into());
    }
    Ok(response)
}

/// Internal struct that Quickbooks returns most
/// of the time when interacting with the API
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(crate) struct QBResponse<T> {
    #[serde(
        alias = "Item",
        alias = "Account",
        alias = "Attachabe",
        alias = "Invoice",
        alias = "Attachable",
        alias = "Bill",
        alias = "CompanyInfo",
        alias = "Customer",
        alias = "Employee",
        alias = "Estimate",
        alias = "Payment",
        alias = "SalesReceipt",
        alias = "Vendor"
    )]
    // TODO : Add more aliases as needed
    object: T,
    time: String,
}

/// Send email of the object to the email given through quickbooks context
pub fn qb_send_email<T: QBItem + QBSendable>(
    item: &T,
    email: &str,
    qb: &QBContext,
    client: &Agent,
) -> Result<T, APIError> {
    let Some(id) = item.id() else {
        return Err(APIErrorInner::NoIdOnSend.into());
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}/{}/send", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        Some([("sendTo", email)]),
    )?;
    #[cfg(feature = "logging")]
    log::info!("Successfully Sent {} object with ID : {}", T::name(), id);
    Ok(response.object)
}
