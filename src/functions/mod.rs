//! # Functions Module
//!
//! This module contains the core functionality for interacting with the QuickBooks API.
//!
//! ## Modules
//!
//! * [`attachment`](attachment) - Handles attachment operations (available with "attachments" feature)
//! * [`create`](create) - Contains functions for creating new objects in QuickBooks
//! * [`delete`](delete) - Provides functionality for deleting objects in QuickBooks
//! * [`pdf`](pdf) - Handles PDF document operations (available with "pdf" feature)
//! * [`query`](query) - Contains functions for querying objects from QuickBooks
//! * [`read`](read) - Provides functionality for retrieving specific objects from QuickBooks
//!
//! This module also contains utility functions for making requests to the QuickBooks API, handling
//! responses, and sending emails through the QuickBooks infrastructure.
use http_client::{http_types::Method, HttpClient};
use quickbooks_types::{QBItem, QBSendable};
// use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

use crate::{error::APIError, QBContext};

pub mod attachment;
pub mod create;
pub mod delete;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod query;
pub mod read;

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
pub(crate) async fn qb_request<'a, T, U, Client>(
    qb: &QBContext,
    client: &Client,
    method: Method,
    path: &str,
    body: Option<&T>,
    content_type: Option<&str>,
    query: Option<&[(&str, &str)]>,
) -> Result<U, APIError>
where
    T: Serialize,
    U: serde::de::DeserializeOwned,
    Client: HttpClient,
{
    let response = qb
        .with_permission(|qb| execute_request(qb, client, method, path, body, content_type, query))
        .await?;
    Ok(response)
}

pub(crate) async fn execute_request<T, U, Client>(
    qb: &QBContext,
    client: &Client,
    method: Method,
    path: &str,
    body: Option<&T>,
    content_type: Option<&str>,
    query: Option<&[(&str, &str)]>,
) -> Result<U, APIError>
where
    T: Serialize,
    U: serde::de::DeserializeOwned,
    Client: HttpClient,
{
    let request = crate::client::build_request(
        method,
        path,
        body,
        query,
        content_type.unwrap_or("application/json"),
        qb.environment,
        &qb.access_token,
    )?;
    let mut response = client.send(request).await?;
    if !response.status().is_success() {
        // panic!("Response: {}", response.body_string().await?);

        return Err(APIError::BadRequest(response.body_json().await?));
    }
    Ok(response.body_json().await?)
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
    object: T,
    time: String,
}

/// Send email of the object to the email given through quickbooks context
pub async fn qb_send_email<T, Client>(
    item: &T,
    email: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError>
where
    T: QBItem + QBSendable,
    Client: HttpClient,
{
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnSend);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::Post,
        &format!("company/{}/{}/{}/send", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        Some(&[("sendTo", email)]),
    )
    .await?;
    log::debug!("Successfully Sent {} object with ID : {}", T::name(), id);
    Ok(response.object)
}
