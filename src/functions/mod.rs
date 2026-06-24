//! Module for `QuickBooks` API functions
//!
//! This module contains functions for interacting with the `QuickBooks` API, including
//! creating, reading, updating, and deleting various `QuickBooks` entities.

use quick_xml::events::Event;
use quickbooks_xsd::types::quick_xml::{WithDeserializer, WithSerializer};
// use quickbooks_types::{QBItem, QBSendable};
use ureq::http::Response;
use ureq::Body;
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    APIResult, QBContext,
};

#[cfg(feature = "attachments")]
pub mod attachment;
#[cfg(feature = "attachments")]
pub use attachment::{content_type_from_ext, empty_attachable, QBUpload};
pub mod create;
// pub mod delete;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod query;
pub mod read;
// pub mod reports;

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
    T: WithSerializer,
    U: WithDeserializer,
    S: AsRef<str>,
    SS: AsRef<str>,
{
    let response = qb.with_permission(|qb| {
        execute_request(qb, client, method, path, body, content_type, query)
    })?;
    let body = response.into_body().into_reader();
    crate::xml_deserialize(body)
}

pub(crate) fn execute_request<S, SS, T: WithSerializer>(
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
        content_type.unwrap_or("application/xml"),
        qb.environment,
        &qb.access_token,
    )?;
    let response = client.run(request)?;
    if !response.status().is_success() {
        return Err(APIErrorInner::BadRequest(crate::xml_deserialize(
            response.into_body().into_reader(),
        )?)
        .into());
    }
    Ok(response)
}
// ---------------------------------------------------------------------------
// Minimal empty type for serialization (used when there is no request body)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) struct Empty;

impl WithSerializer for Empty {
    type Serializer<'x> = Empty;
    fn serializer<'ser>(
        &'ser self,
        _: Option<&'ser str>,
        _: bool,
    ) -> Result<Self, quickbooks_xsd::types::quick_xml::Error> {
        Ok(Empty)
    }
}

impl<'ser> quickbooks_xsd::types::quick_xml::Serializer<'ser> for Empty {
    fn next(
        &mut self,
        _: &mut quickbooks_xsd::types::quick_xml::SerializeHelper,
    ) -> Option<Result<Event<'ser>, quickbooks_xsd::types::quick_xml::Error>> {
        None
    }
}
