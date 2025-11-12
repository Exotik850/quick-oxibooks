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
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let qb_context = todo!();
//! # let client = reqwest::Client::new();
//! let attachment = Attachable {
//!     file_name: Some("invoice.pdf".into()),
//!     note: Some("Invoice attachment".into()),
//!     ..Default::default()
//! };
//!
//! let uploaded = attachment.upload(&qb_context, &Agent)?;
//! # Ok(())
//! # }
//! ```
use base64::Engine;
use quickbooks_types::{Attachable, QBAttachable};
use ureq::{
    http::{request::Builder, Request, StatusCode},
    Agent,
};

use crate::{
    error::{APIError, APIErrorInner, QBErrorResponse},
    APIResult, QBContext,
};

const BOUNDARY: &str = "----------------quick-oxibooks"; // Multipart boundary for the request

/// Trait for uploading file attachments to QuickBooks.
///
/// This trait provides the `upload` method for sending file attachments to QuickBooks
/// and linking them to specific entities (invoices, customers, vendors, etc.).
/// It validates that attachments meet upload requirements before sending.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for the [`Attachable`] type from `quickbooks_types`.
/// You don't need to implement it manually.
///
/// # Upload Requirements
///
/// Before uploading, attachments must have:
/// - Valid `file_name` pointing to an existing file
/// - Either a `note` description or both `file_name` and content
/// - Proper `attachable_ref` linking to a QuickBooks entity (optional for standalone attachments)
///
/// # Supported File Types
///
/// QuickBooks supports various file types including:
/// - Documents: PDF, DOC, DOCX, TXT
/// - Images: JPG, PNG, GIF, BMP
/// - Spreadsheets: XLS, XLSX, CSV
/// - Other: ZIP, and more
///
/// # Examples
///
/// ## Upload a Receipt Image
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::QBUpload};
/// use quickbooks_types::{Attachable, AttachmentCategory};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Create an attachment for a receipt
/// let mut attachment = Attachable::default();
/// attachment.file_name = Some("receipt_123.pdf".to_string());
/// attachment.note = Some("Restaurant receipt for business lunch".to_string());
/// attachment.category = Some(AttachmentCategory::Receipt);
///
/// // Upload to QuickBooks
/// let uploaded = attachment.upload(&qb_context, &client)?;
/// println!("Uploaded attachment with ID: {:?}", uploaded.id());
/// ```
///
/// ## Upload and Link to Invoice
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::{QBUpload, QBQuery}};
/// use quickbooks_types::{Attachable, AttachableRef, Invoice, QBToAttachableRef};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Find the invoice to attach to
/// let invoice = Invoice::query_single("WHERE DocNumber = 'INV-001'", &qb_context, &client)?;
///
/// // Create attachment linked to invoice
/// let mut attachment = Attachable::default();
/// attachment.file_name = Some("supporting_docs.pdf".to_string());
/// attachment.note = Some("Supporting documentation for invoice".to_string());
/// attachment.attachable_ref = Some(vec![invoice.to_attachable_ref()?]);
///
/// // Upload with link
/// let uploaded = attachment.upload(&qb_context, &client)?;
/// ```
///
/// ## Upload Company Logo
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::QBUpload};
/// use quickbooks_types::{Attachable, AttachmentCategory};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Upload a company logo
/// let mut logo = Attachable::default();
/// logo.file_name = Some("company_logo.png".to_string());
/// logo.note = Some("Company logo for letterhead".to_string());
/// logo.category = Some(AttachmentCategory::Image);
///
/// let uploaded = logo.upload(&qb_context, &client)?;
/// ```
///
/// # Return Value
///
/// Returns the uploaded attachment with QuickBooks-assigned ID and metadata.
/// The returned attachment can be used to reference the file in other operations.
///
/// # File Path Resolution
///
/// The `file_name` field should contain:
/// - Absolute path to the file: `/path/to/file.pdf`
/// - Relative path from current directory: `./documents/receipt.jpg`
/// - Just filename if in current directory: `document.pdf`
///
/// # Errors
///
/// - `AttachableUploadMissingItems`: Missing required fields (file_name or note)
/// - `InvalidFile`: File doesn't exist or invalid file extension
/// - `IoError`: File reading errors
/// - `UreqError`: Network or HTTP errors during upload
/// - `BadRequest`: QuickBooks rejected the attachment (size, type, etc.)
/// - `NoAttachableObjects`: Upload succeeded but no attachment in response
///
/// # File Size Limits
///
/// QuickBooks has file size limits (typically 100MB per file). Large files
/// may take longer to upload and could timeout on slow connections.
pub trait QBUpload {
    /// Uploads the attachment
    /// returns an error if the attachment is not suitable for upload
    /// or if the request itself fails
    fn upload(&self, qb: &QBContext, client: &Agent) -> APIResult<Self>
    where
        Self: Sized;
}

impl QBUpload for Attachable {
    fn upload(&self, qb: &QBContext, client: &Agent) -> APIResult<Self> {
        qb_upload(self, qb, client)
    }
}

/// Attach a file to another Quickbooks objct
/// via a `Attachable` object
///
/// Uploads the file and makes the `attachable` object
/// in `QuickBooks`.
fn qb_upload(attachable: &Attachable, qb: &QBContext, client: &Agent) -> APIResult<Attachable> {
    attachable.can_upload()?;

    let request = make_upload_request(attachable, qb)?;

    let mut qb_response: AttachableResponseExt = qb.with_permission(|_| {
        let response = client.run(request)?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            // Handle rate limiting by QuickBooks
            return Err(APIErrorInner::ThrottleLimitReached.into());
        }
        if !response.status().is_success() {
            return Err(APIErrorInner::BadRequest(response.into_body().read_json()?).into());
        }
        let out = response.into_body().read_json()?;
        Ok(out)
    })?;

    if qb_response.ar.is_empty() {
        return Err(APIErrorInner::NoAttachableObjects.into());
    }

    let obj = match qb_response.ar.swap_remove(0) {
        AttachableResponse::Fault(fault) => {
            return Err(APIErrorInner::BadRequest(QBErrorResponse {
                fault: Some(fault),
                ..Default::default()
            })
            .into())
        }
        AttachableResponse::Attachable(attachable) => attachable,
    };

    #[cfg(feature = "logging")]
    log::debug!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

    Ok(obj)
}

fn make_upload_request(attachable: &Attachable, qb: &QBContext) -> APIResult<Request<String>> {
    let path = format!("company/{}/upload", qb.company_id);
    let url = crate::client::build_url(
        qb.environment,
        &path,
        None::<std::iter::Empty<(&str, &str)>>,
    );
    let mut request = Request::post(url.as_str());
    request = crate::client::set_headers("multipart/form-data", &qb.access_token, request);
    let request = make_multipart(request, attachable)?;
    Ok(request)
}

fn make_multipart(req: Builder, attachable: &Attachable) -> Result<Request<String>, APIError> {
    let file_path = attachable
        .file_path
        .as_deref()
        .ok_or_else(|| APIErrorInner::AttachableUploadMissingItems("file_path"))?;
    let ct = attachable
        .content_type
        .as_deref()
        .ok_or_else(|| APIErrorInner::AttachableUploadMissingItems("content_type"))?;
    let mut body = String::new();

    body.push_str(&format!("--{BOUNDARY}\r\n"));

    body.push_str("Content-Disposition: form-data; name=\"file_metadata_01\"\r\n");
    body.push_str("Content-Type: application/json\r\n\r\n");

    let json_body = serde_json::to_string(attachable)?;
    body.push_str(&json_body);
    body.push_str("\r\n");

    let file_content = std::fs::read(file_path)?;
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(file_content);
    body.push_str(&format!("--{BOUNDARY}\r\n"));

    // let sep = if file_path.contains('\\') { '\\' } else { '/' };
    // let file_name = file_path.split(sep).last().unwrap_or(file_path);
    let file_name = file_path
        .file_name()
        .ok_or_else(|| APIErrorInner::InvalidFile(file_path.to_string_lossy().to_string()))?
        .to_string_lossy();

    body.push_str(&format!(
        "Content-Disposition: form-data; name=\"file_content_01\"; filename=\"{file_name}\"\r\n"
    ));
    body.push_str(&format!("Content-Type: {ct}\r\n"));
    body.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
    body.push_str(&encoded);
    body.push_str("\r\n");

    body.push_str(&format!("--{BOUNDARY}--\r\n"));

    let content = format!("multipart/form-data; boundary={BOUNDARY}");
    Ok(req
        .header("Content-Type", content)
        .header("Content-Length", body.len().to_string())
        .body(body)?)
}

// fn get_ext(input: &str) -> Option<&str> {
//     let path = Path::new(input);
//     path.extension().and_then(|ext| ext.to_str())
// }

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
    Fault(crate::error::Fault),
}
