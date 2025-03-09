use quickbooks_types::{QBItem, QBSendable};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

use crate::{error::APIError, QBContext};

#[cfg(feature = "attachments")]
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
pub(crate) async fn qb_request<'a, T, U>(
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
{
    let response = qb
        .with_permission(|qb| execute_request(qb, client, method, path, body, content_type, query))
        .await?;
    Ok(response.json().await?)
}

pub(crate) async fn execute_request<T: Serialize>(
    qb: &QBContext,
    client: &Client,
    method: Method,
    path: &str,
    body: Option<&T>,
    content_type: Option<&str>,
    query: Option<&[(&str, &str)]>,
) -> Result<reqwest::Response, APIError> {
    let request = crate::client::build_request(
        method,
        path,
        body,
        query,
        content_type.unwrap_or("application/json"),
        qb.environment,
        client,
        &qb.access_token,
    )?;
    let response = client.execute(request).await?;
    if !response.status().is_success() {
        return Err(APIError::BadRequest(response.json().await?));
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
    object: T,
    time: String,
}

/// Send email of the object to the email given through quickbooks context
pub async fn qb_send_email<T: QBItem + QBSendable>(
    item: &T,
    email: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnSend);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        reqwest::Method::POST,
        &format!("company/{}/{}/{}/send", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        Some(&[("sendTo", email)]),
    )
    .await?;
    log::info!("Successfully Sent {} object with ID : {}", T::name(), id);
    Ok(response.object)
}
