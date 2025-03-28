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

use std::path::Path;

use base64::Engine;
use http_client::{HttpClient, Request};
use quickbooks_types::{content_type_from_ext, Attachable, QBAttachable};

use crate::{
    error::{APIError, Fault, QBErrorResponse}, QBContext
};

const BOUNDARY: &str = "----------------quick-oxibooks"; // Multipart boundary for the request

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
//         return Err(APIError::InvalidFile(extension.to_string()));
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
    attachable.can_upload()?;

    let request = make_upload_request(attachable, qb).await?;

    let mut qb_response: AttachableResponseExt = qb
        .with_permission(|_| async {
            let mut response = client.send(request).await?;
            if !response.status().is_success() {
                return Err(APIError::BadRequest(response.body_json().await?));
            }
            let out = response.body_json().await?;
            Ok(out)
        })
        .await?;

    if qb_response.ar.is_empty() {
        return Err(APIError::NoAttachableObjects);
    };

    let obj = match qb_response.ar.swap_remove(0) {
        AttachableResponse::Fault(fault) => {
            return Err(APIError::BadRequest(QBErrorResponse {
                fault: Some(fault),
                ..Default::default()
            }))
        }
        AttachableResponse::Attachable(attachable) => attachable,
    };

    log::info!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

    Ok(obj)
}

async fn make_upload_request(attachable: &Attachable, qb: &QBContext) -> Result<Request, APIError> {
    let path = format!("company/{}/upload", qb.company_id);
    let url = crate::client::build_url(qb.environment, &path, Some(&[]))?;
    let mut request = Request::post(url);
    crate::client::set_headers("multipart/form-data", &qb.access_token, &mut request);
    make_multipart(&mut request, attachable).await?;
    Ok(request)
}

async fn make_multipart(req: &mut Request, attachable: &Attachable) -> Result<(), APIError> {
    let file_path = attachable
        .file_path
        .as_deref()
        .ok_or_else(|| APIError::AttachableUploadMissingItems("file_path"))?;
    let ct = attachable
        .content_type
        .as_deref()
        .ok_or_else(|| APIError::AttachableUploadMissingItems("content_type"))?;
    let mut body = String::new();

    body.push_str(&format!("--{BOUNDARY}\r\n"));

    body.push_str(&format!(
        "Content-Disposition: form-data; name=\"file_metadata_01\"\r\n"
    ));
    body.push_str("Content-Type: application/json\r\n\r\n");

    let json_body = serde_json::to_string(attachable)?;
    body.push_str(&json_body);
    body.push_str("\r\n");

    let file_content = async_fs::read(file_path).await?;
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(file_content);
    body.push_str(&format!("--{BOUNDARY}\r\n"));

    // let sep = if file_path.contains('\\') { '\\' } else { '/' };
    // let file_name = file_path.split(sep).last().unwrap_or(file_path);
    let file_name = file_path
        .file_name()
        .ok_or_else(|| APIError::InvalidFile(file_path.to_string_lossy().to_string()))?
        .to_string_lossy();

    body.push_str(&format!(
        "Content-Disposition: form-data; name=\"file_content_01\"; filename=\"{}\"\r\n",
        file_name
    ));
    body.push_str(&format!("Content-Type: {ct}\r\n"));
    body.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
    body.push_str(&encoded);
    body.push_str("\r\n");

    body.push_str(&format!("--{BOUNDARY}--\r\n"));

    let content = format!("multipart/form-data; boundary={BOUNDARY}");
    req.insert_header("Content-Type", content);
    req.insert_header("Content-Length", body.len().to_string());
    req.set_body(body);

    Ok(())
}

fn get_ext(input: &str) -> Option<&str> {
    let path = Path::new(input);
    path.extension().and_then(|ext| ext.to_str())
}

#[derive(Debug, serde::Deserialize)]
struct AttachableResponseExt {
    #[serde(rename = "AttachableResponse")]
    ar: Vec<AttachableResponse>,
    #[allow(dead_code)]
    time: String,
}

#[derive(serde::Deserialize, Debug)]
enum AttachableResponse {
    Attachable(Attachable),
    Fault(Fault),
}
