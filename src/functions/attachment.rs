//! Module for handling attachments in QuickBooks Online
//!
//! This module provides functionality for uploading files as attachments
//! to QuickBooks Online objects. It handles the file encoding, metadata,
//! and multipart form upload process.
//!
//! # Example
//!
//! ```no_run
//! use quickbooks_xsd::finance::AttachableType as Attachable;
//! use quickbooks_xsd::intuit_rest_service_def::AttachableResponseType;
//! use quick_oxibooks::functions::attachment::QBUpload;
//! use quick_oxibooks::{QBContext, Environment};
//! use ureq::Agent;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let qb_context = todo!();
//! # let client = Agent::new_with_defaults();
//! let attachment = Attachable {
//!     file_name: Some("invoice.pdf".into()),
//!     note: Some("Invoice attachment".into()),
//!     content_type: Some("application/pdf".into()),
//!     ..quick_oxibooks::functions::attachment::empty_attachable()
//! };
//!
//! let uploaded = attachment.upload(&qb_context, &client)?;
//! # Ok(())
//! # }
//! ```
use base64::Engine;
use std::path::Path;

use quickbooks_xsd::{
    finance::AttachableType as Attachable,
    intuit_rest_service_def::{AttachableResponseType as AttachableResponse, IntuitResponse},
    types::quick_xml::SerializeSync,
};
use ureq::{
    http::{request::Builder, Request, StatusCode},
    Agent,
};

use crate::{
    error::{APIError, APIErrorInner},
    APIResult, QBContext,
};

const BOUNDARY: &str = "----------------quick-oxibooks"; // Multipart boundary for the request

/// Constructs an [`Attachable`] with every field set to its empty/default value.
///
/// `AttachableType` is generated from the QuickBooks XSD and does not derive
/// `Default`, so this helper provides an ergonomic way to create a blank
/// attachment that can be customized with struct-update syntax:
///
/// ```no_run
/// use quickbooks_xsd::finance::AttachableType as Attachable;
/// use quick_oxibooks::functions::attachment::empty_attachable;
///
/// let attachment = Attachable {
///     file_name: Some("invoice.pdf".into()),
///     content_type: Some("application/pdf".into()),
///     ..empty_attachable()
/// };
/// ```
#[must_use]
pub fn empty_attachable() -> Attachable {
    Attachable {
        domain: None,
        status: None,
        sparse: None,
        id: None,
        sync_token: None,
        meta_data: None,
        custom_field: Vec::new(),
        attachable_ref: Vec::new(),
        file_name: None,
        file_access_uri: None,
        temp_download_uri: None,
        size: None,
        content_type: None,
        category: None,
        lat: None,
        long: None,
        place_name: None,
        note: None,
        tag: None,
        thumbnail_file_access_uri: None,
        thumbnail_temp_download_uri: None,
        attachable_ex: None,
        document_id: None,
    }
}

/// Trait for uploading file attachments to QuickBooks.
///
/// This trait provides the `upload` method for sending file attachments to QuickBooks
/// and linking them to specific entities (invoices, customers, vendors, etc.).
/// It validates that attachments meet upload requirements before sending.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for the [`Attachable`] type from `quickbooks_xsd`.
/// You don't need to implement it manually.
///
/// # Upload Requirements
///
/// Before uploading, attachments must have:
/// - A `file_name` pointing to an existing, readable file on disk
/// - A `content_type` describing the MIME type of the file (see [`content_type_from_ext`])
/// - Either a `note` description or both `file_name` and content (file content is
///   always required for an upload; a `note`-only attachment is not uploadable)
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
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment, functions::attachment::{QBUpload, empty_attachable}};
/// use quickbooks_xsd::finance::AttachableType as Attachable;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
///
/// // Create an attachment for a receipt
/// let attachment = Attachable {
///     file_name: Some("receipt_123.pdf".to_string()),
///     note: Some("Restaurant receipt for business lunch".to_string()),
///     content_type: Some("application/pdf".to_string()),
///     category: Some("Receipt".to_string()),
///     ..empty_attachable()
/// };
///
/// // Upload to QuickBooks
/// let uploaded = attachment.upload(&qb_context, &client).unwrap();
/// println!("Uploaded attachment with ID: {:?}", uploaded.id);
/// ```
///
/// ## Upload and Link to Invoice
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment, functions::attachment::{QBUpload, empty_attachable}, traits::QBToAttachableRef};
/// use quick_oxibooks::functions::query::QBQuery;
/// use quickbooks_xsd::finance::AttachableType as Attachable;
/// use quickbooks_xsd::intuit_rest_service_def::Invoice;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
///
/// // Find the invoice to attach to
/// let invoice = Invoice::query_single("WHERE DocNumber = 'INV-001'", &qb_context, &client).unwrap().unwrap();
///
/// // Create attachment linked to invoice
/// let attachment = Attachable {
///     file_name: Some("supporting_docs.pdf".to_string()),
///     note: Some("Supporting documentation for invoice".to_string()),
///     content_type: Some("application/pdf".to_string()),
///     attachable_ref: vec![invoice.to_attachable_ref().unwrap()],
///     ..empty_attachable()
/// };
///
/// // Upload with link
/// let uploaded = attachment.upload(&qb_context, &client).unwrap();
/// ```
///
/// ## Upload Company Logo
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment, functions::attachment::{QBUpload, empty_attachable}};
/// use quickbooks_xsd::finance::AttachableType as Attachable;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
///
/// // Upload a company logo
/// let logo = Attachable {
///     file_name: Some("company_logo.png".to_string()),
///     note: Some("Company logo for letterhead".to_string()),
///     content_type: Some("image/png".to_string()),
///     category: Some("Image".to_string()),
///     ..empty_attachable()
/// };
///
/// let uploaded = logo.upload(&qb_context, &client).unwrap();
/// ```
///
/// # Return Value
///
/// Returns the uploaded attachment with QuickBooks-assigned ID and metadata.
/// The returned attachment can be used to reference the file in other operations.
///
/// # File Path Resolution
///
/// The `file_name` field is treated as a filesystem path for the purpose of reading
/// the file bytes to upload. Only the basename (the final path component) is sent to
/// QuickBooks as the attachment's `FileName`, so the on-disk path is never leaked.
/// The path may be:
/// - Absolute path to the file: `/path/to/file.pdf`
/// - Relative path from current directory: `./documents/receipt.jpg`
/// - Just filename if in current directory: `document.pdf`
///
/// # Errors
///
/// - `CreateMissingItems`: Missing required fields (`file_name`, `note`, or `content_type`)
/// - `InvalidFile`: File doesn't exist, is not a regular file, or has no usable basename
/// - `IoError`: File reading errors
/// - `UreqError`: Network or HTTP errors during upload
/// - `BadRequest`: QuickBooks rejected the attachment (size, type, etc.)
/// - `AttachableFault`: QuickBooks returned a `Fault` in the upload response
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

fn ext(path: &str) -> Option<&str> {
    Path::new(path).extension().and_then(|os| os.to_str())
}

fn can_upload(attachable: &Attachable) -> APIResult<()> {
    let mut missing = Vec::new();
    if attachable.file_name.is_none() {
        missing.push("file_name");
    }
    if attachable.content_type.is_none()
        && attachable
            .file_name
            .as_ref()
            .is_some_and(|f| content_type_from_ext(ext(&f).unwrap_or("")).is_none())
    {
        missing.push("content_type");
    }
    // A note-only attachment is not uploadable: there is no file content to send.
    // QuickBooks' upload endpoint always expects a file part.
    if attachable.note.is_none() && attachable.file_name.is_none() {
        missing.push("note");
    }
    if !missing.is_empty() {
        return Err(APIErrorInner::CreateMissingItems(missing).into());
    }

    // Safe to unwrap: presence was checked above.
    let file_name = attachable.file_name.as_deref().unwrap();
    let path = Path::new(file_name);
    if !path.exists() {
        println!("File does not exist: {}", file_name);
        return Err(APIErrorInner::InvalidFile(file_name.to_string()).into());
    }
    if !path.is_file() {
        println!("File is not a regular file: {}", file_name);
        return Err(APIErrorInner::InvalidFile(file_name.to_string()).into());
    }
    // Ensure the basename can be derived; QuickBooks rejects empty file names.
    if file_basename(path).is_none() {
        println!("File has no usable basename: {}", file_name);
        return Err(APIErrorInner::InvalidFile(file_name.to_string()).into());
    }
    Ok(())
}

/// Extracts the basename (final path component) from a path, returning `None`
/// if the path has no usable file name component.
fn file_basename(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|os| os.to_str())
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string)
}

/// Derives a MIME content type from a file extension.
///
/// Returns `None` for unrecognized extensions. The list mirrors the file types
/// accepted by the QuickBooks Online attachable upload endpoint.
#[must_use]
pub fn content_type_from_ext(ext: &str) -> Option<&'static str> {
    let out = match ext.to_ascii_lowercase().as_str() {
        "ai" | "eps" => "application/postscript",
        "csv" => "text/csv",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "gif" => "image/gif",
        "jpeg" | "jpg" => "image/jpeg",
        "png" => "image/png",
        "rtf" => "text/rtf",
        "txt" => "text/plain",
        "tif" | "tiff" => "image/tiff",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "pdf" => "application/pdf",
        "xls" => "application/vnd.ms-excel",
        "xml" => "text/xml",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => return None,
    };
    Some(out)
}

/// Attach a file to another Quickbooks objct
/// via a `Attachable` object
///
/// Uploads the file and makes the `attachable` object
/// in `QuickBooks`.
fn qb_upload(attachable: &Attachable, qb: &QBContext, client: &Agent) -> APIResult<Attachable> {
    can_upload(attachable)?;

    let request = make_upload_request(attachable, qb)?;

    let mut qb_response: IntuitResponse = qb.with_permission(|_| {
        let response = client.run(request)?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            // Handle rate limiting by QuickBooks
            return Err(APIErrorInner::ThrottleLimitReached.into());
        }
        if !response.status().is_success() {
            return Err(APIErrorInner::BadRequest(crate::xml_deserialize(
                response.into_body().into_reader(),
            )?)
            .into());
        }
        let out = crate::xml_deserialize(response.into_body().into_reader())?;
        Ok(out)
    })?;

    // let obj = qb_response
    //     .attachable_response_mut()
    //     .ok_or_else(|| APIErrorInner::NoAttachableObjects)?
    //     .content
    //     .as_mut()
    //     .ok_or_else(|| APIErrorInner::NoAttachableObjects)?;

    if let Some(err) = qb_response.fault_mut() {
        return Err(APIErrorInner::AttachableFault(err.clone()).into());
    }

    let Some(obj) = qb_response
        .attachable_response_mut()
        .and_then(|ar| ar.content.as_mut())
    else {
        return Err(APIErrorInner::NoAttachableObjects.into());
    };

    let obj = match obj {
        quickbooks_xsd::intuit_rest_service_def::AttachableResponseTypeContent::Attachable(
            attachable_type,
        ) => std::mem::replace(attachable_type, empty_attachable()),
        quickbooks_xsd::intuit_rest_service_def::AttachableResponseTypeContent::Fault(
            fault_type,
        ) => {
            return Err(APIErrorInner::AttachableFault(fault_type.clone()).into());
        }
    };

    #[cfg(feature = "logging")]
    log::debug!("Sent attachment : {:?}", obj.file_name);

    Ok(obj)
}

fn make_upload_request(attachable: &Attachable, qb: &QBContext) -> APIResult<Request<Vec<u8>>> {
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

fn make_multipart(req: Builder, attachable: &Attachable) -> Result<Request<Vec<u8>>, APIError> {
    // All of these are guaranteed by `can_upload`, which is always called first.
    let file_path_str = attachable.file_name.as_deref().unwrap();
    let ext = Path::new(file_path_str)
        .extension()
        .and_then(|os| os.to_str())
        .ok_or_else(|| APIErrorInner::InvalidFile(file_path_str.to_string()))?;
    let ct = attachable
        .content_type
        .as_deref()
        .or_else(|| content_type_from_ext(ext))
        .unwrap_or("application/octet-stream");
    let file_path = Path::new(file_path_str);

    // QuickBooks expects only the basename in the metadata `FileName` field and in
    // the multipart `filename=` parameter — never the on-disk path. Build a shallow
    // copy of the attachable with `file_name` set to the basename so the serialized
    // XML metadata does not leak the local filesystem path.
    let basename = file_basename(file_path)
        .ok_or_else(|| APIErrorInner::InvalidFile(file_path_str.to_string()))?;

    let mut metadata = attachable.clone();
    metadata.file_name = Some(basename.clone());
    metadata.content_type = Some(ct.to_string());

    let mut body = Vec::new();

    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());

    body.extend_from_slice(
        "Content-Disposition: form-data; name=\"file_metadata_01\"\r\n".as_bytes(),
    );
    body.extend_from_slice("Content-Type: application/xml\r\n\r\n".as_bytes());

    metadata.serialize("Attachable", &mut quick_xml::Writer::new(&mut body))?;
    body.extend_from_slice(b"\r\n");

    let file_content = std::fs::read(file_path)?;
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(file_content);
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());

    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file_content_01\"; filename=\"{basename}\"\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {ct}\r\n").as_bytes());
    body.extend_from_slice(b"Content-Transfer-Encoding: base64\r\n\r\n");
    body.extend_from_slice(encoded.as_bytes());
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());

    let content = format!("multipart/form-data; boundary={BOUNDARY}");
    Ok(req
        .header("Content-Type", content)
        .header("Content-Length", body.len().to_string())
        .body(body)?)
}

// #[derive(Debug, serde::Deserialize)]
// struct AttachableResponseExt {
//     #[serde(rename = "AttachableResponse")]
//     ar: Vec<AttachableResponse>,
//     #[allow(dead_code)]
//     time: String,
// }

// #[derive(serde::Deserialize, Debug)]
// enum AttachableResponse {
//     Attachable(Attachable),
//     Fault(crate::error::Fault),
// }
