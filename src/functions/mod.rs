//! Module for `QuickBooks` API functions
//!
//! This module contains functions for interacting with the `QuickBooks` API, including
//! creating, reading, updating, and deleting various `QuickBooks` entities.

use quick_xml::events::Event;
use quickbooks_xsd::types::quick_xml::{Deserializer, WithDeserializer, WithSerializer};
// use quickbooks_types::{QBItem, QBSendable};
use serde::{Deserialize, Serialize};
use ureq::http::Response;
use ureq::Body;
use ureq::{http::Method, Agent};

use crate::functions::query::QBDeError;
use crate::traits::QBItem;
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
// #[cfg(feature = "pdf")]
// pub mod pdf;
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

// /// Internal struct that Quickbooks returns most
// /// of the time when interacting with the API
// #[derive(Debug, Clone, Default)]
// pub(crate) struct QBResponse<T> {
//     object: T,
//     time: String,
// }

// impl<T: QBItem> WithDeserializer for QBResponse<T> {
//     type Deserializer = QBResponseDe<T>;
// }

// #[derive(Debug)]
// pub struct QBResponseDe<T: QBItem> {
//     deserializer: Option<T::Deserializer>,
//     time: Option<String>,
//     state: QBResponseDeState,
// }

// #[derive(Debug)]
// enum QBResponseDeState {
//     Start,
//     InItem,
//     End,
// }

// /*
//   <IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2024-06-01T12:00:00Z">
//     <Customer>
//       <Id>123</Id>
//       <SyncToken>0</SyncToken>
//       <MetaData>
//         <CreateTime>2024-06-01T12:00:00Z</CreateTime>
//         <LastUpdatedTime>2024-06-01T12:00:00Z</LastUpdatedTime>
//       </MetaData>
//       <!-- Other customer fields -->
//     </Customer>
//   </IntuitResponse>
// */

// impl<'de, T: QBItem> Deserializer<'de, QBResponse<T>> for QBResponseDe<T> {
//     fn init(
//         helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper,
//         event: quick_xml::events::Event<'de>,
//     ) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QBResponse<T>> {
//         todo!()
//     }

//     fn next(
//         self,
//         helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper,
//         event: quick_xml::events::Event<'de>,
//     ) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QBResponse<T>> {
//         todo!()
//     }

//     fn finish(
//         self,
//         helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper,
//     ) -> Result<QBResponse<T>, quickbooks_xsd::types::quick_xml::Error> {
//         todo!()
//     }
// }

// // fn parse_qb_response<T: QBItem>(reader: impl std::io::Read) -> APIResult<T> {

// // }

// // /// Send email of the object to the email given through quickbooks context
// // pub fn qb_send_email<T: QBItem + QBSendable>(
// //     item: &T,
// //     email: &str,
// //     qb: &QBContext,
// //     client: &Agent,
// // ) -> Result<T, APIError> {
// //     let Some(id) = item.id() else {
// //         return Err(APIErrorInner::NoIdOnSend.into());
// //     };

// //     let response: QBResponse<T> = qb_request(
// //         qb,
// //         client,
// //         Method::POST,
// //         &format!("company/{}/{}/{}/send", qb.company_id, T::QB_ID, id),
// //         None::<&()>,
// //         None,
// //         Some([("sendTo", email)]),
// //     )?;
// //     #[cfg(feature = "logging")]
// //     log::info!("Successfully Sent {} object with ID : {}", T::name(), id);
// //     Ok(response.object)
// // }
