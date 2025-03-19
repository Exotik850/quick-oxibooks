//! Module for handling attachments in QuickBooks Online
//!
//! This module provides functionality for uploading files as attachments
//! to QuickBooks Online objects. It handles the file encoding, metadata,
//! and multipart form upload process.
//!
//! # Example
//!
//! ```rust
//! use quickbooks_types::Attachable;
//! use quick_oxibooks::functions::attachment::QBUpload;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let qb_context = todo!();
//! # let client = reqwest::Client::new();
//! let attachment = Attachable {
//!     file_name: Some("invoice.pdf".into()),
//!     note: Some("Invoice attachment".into()),
//!     ..Default::default()
//! };
//!
//! let uploaded = attachment.upload(&qb_context, &client).await?;
//! # Ok(())
//! # }
//! ```

use base64::Engine;
use http_client::{HttpClient, Request};
use quickbooks_types::{content_type_from_ext, Attachable, QBAttachable};
use std::path::{Path, PathBuf};
// use reqwest::{
//     header::{self, HeaderValue},
//     multipart::{Form, Part},
//     Client, Method, Request,
// };

use crate::{error::APIError, QBContext};

// async fn _make_file_part(file_name: impl AsRef<Path>) -> Result<Part, APIError> {
//     let buf = async_fs::read(&file_name).await?;
//     let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf);

//     let file_headers = {
//         let mut headers = header::HeaderMap::new();
//         headers.append(
//             "Content-Transfer-Encoding",
//             HeaderValue::from_static("base64"),
//         );
//         headers
//     };

//     // Would've returned an error already if it was directory, safe to unwrap
//     let ext: PathBuf = file_name.as_ref().to_path_buf();
//     let extension = ext.extension().unwrap().to_str().unwrap();
//     let Some(ct) = content_type_from_ext(extension) else {
//         return Err(APIError::InvalidFileExtension(extension.to_string()));
//     };

//     let file_part = Part::bytes(encoded.into_bytes())
//         .mime_str(ct)?
//         .file_name(
//             file_name
//                 .as_ref()
//                 .file_name()
//                 .unwrap()
//                 .to_string_lossy()
//                 .to_string(),
//         )
//         .headers(file_headers);

//     Ok(file_part)
// }

/// Trait for uploading an attachment
pub trait QBUpload {
    /// Uploads the attachment
    /// returns an error if the attachment is not suitable for upload
    /// or if the request itself fails
    fn upload<Client: HttpClient>(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Self, APIError>>
    where
        Self: Sized;
}

impl QBUpload for Attachable {
    fn upload<Client: HttpClient>(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Self, APIError>> {
        qb_upload(self, qb, client)
    }
}

/// Attach a file to another Quickbooks objct
/// via a `Attachable` object
///
/// Uploads the file and makes the `attachable` object
/// in QuickBooks.
async fn qb_upload<Client: HttpClient>(
    attachable: &Attachable,
    qb: &QBContext,
    client: &Client,
) -> Result<Attachable, APIError> {
    if !attachable.can_upload() {
        return Err(APIError::AttachableUploadMissingItems);
    }

    let request = make_upload_request(attachable, qb).await?;

    let mut qb_response: AttachableResponseExt = qb
        .with_permission(|_| async {
            let mut response = client.send(request).await?;
            let out = response.body_json().await?;
            Ok(out)
        })
        .await?;

    // if !response.status().is_success() {
    //     return Err(APIError::BadRequest(response.json().await?));
    // }

    // let mut qb_response: AttachableResponseExt = response.json().await?;
    if qb_response.ar.is_empty() {
        return Err(APIError::NoAttachableObjects);
    };

    let obj = qb_response.ar.swap_remove(0).attachable;

    log::info!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

    Ok(obj)
}

async fn make_upload_request(attachable: &Attachable, qb: &QBContext) -> Result<Request, APIError> {
    let file_name = attachable
        .file_name
        .as_ref()
        .ok_or(APIError::AttachableUploadMissingItems)?;

    let path = format!("company/{}/upload", qb.company_id);
    let url = crate::client::build_url(qb.environment, &path, Some(&[]))?;
    let mut request = Request::post(url);
    crate::client::set_headers("application/pdf", &qb.access_token, &mut request);
    // let request_headers = crate::client::build_headers("application/pdf", &qb.access_token)?;
    let mut multipart = http_client_multipart::Multipart::new();
    let json_body = serde_json::to_string(attachable).expect("Couldn't Serialize Attachment");
    multipart.add_text("file_metadata_01", json_body);
    multipart
        .add_file(
            "file_content_01",
            file_name,
            Some(http_client_multipart::ContentTransferEncoding::Base64),
        )
        .await?;

    multipart.set_request(&mut request).await?;

    Ok(request)
}

#[derive(Debug, serde::Deserialize)]
struct AttachableResponseExt {
    #[serde(rename = "AttachableResponse")]
    ar: Vec<AttachableResponse>,
    #[allow(dead_code)]
    time: String,
}

#[derive(serde::Deserialize, Debug)]
struct AttachableResponse {
    #[serde(rename = "Attachable")]
    attachable: Attachable,
}
