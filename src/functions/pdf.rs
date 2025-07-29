//! # PDF Generation Module
//!
//! This module provides functionality for generating and saving PDF documents from QuickBooks entities.
//!
//! It includes traits and functions to:
//! - Retrieve PDF bytes for QuickBooks entities
//! - Save PDFs directly to files
//!
//! ## Features
//!
//! - Async PDF generation
//! - Direct file saving
//! - Automatic implementation for all types that implement `QBItem` and `QBPDFable`
//!
//! ## Example
//!
//! ```
//! use quick_oxibooks::{QBContext, Environment};
//! use quickbooks_types::{Invoice, QBGetPDF};
//! use reqwest::Client;
//!
//! #[tokio::main]
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Setup QuickBooks context
//!     let qb_context = QBContext::new(
//!         "company_id".to_string(),
//!         "access_token".to_string(),
//!         Environment::Production,
//!     );
//!     
//!     let client = Client::new();
//!     
//!     // Get invoice (assuming you have retrieved it from QuickBooks API)
//!     let invoice = Invoice::new();
//!     
//!     // Save invoice PDF to file
//!     invoice.save_pdf_to_file("invoice.pdf", &qb_context, &Agent)?;
//!     
//!     // Alternatively, get PDF bytes
//!     let pdf_bytes = invoice.get_pdf_bytes(&qb_context, &Agent)?;
//!     
//!     Ok(())
//! }
//! ```
use std::io::Write;

use quickbooks_types::{QBItem, QBPDFable};
use ureq::Agent;

use crate::{
    error::{APIError, APIErrorInner},
    APIResult, Environment, QBContext,
};

/// Trait for generating PDF documents from QuickBooks entities.
///
/// This trait provides methods for retrieving PDF representations of QuickBooks
/// entities such as invoices, estimates, sales receipts, and other printable documents.
/// PDFs are generated server-side by QuickBooks and returned as binary data.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement both
/// [`QBItem`] and [`QBPDFable`]. You don't need to implement it manually.
///
/// # Supported Entities
///
/// Not all QuickBooks entities support PDF generation. Typically supported:
/// - **Invoices**: Customer-facing invoices
/// - **Estimates**: Price quotes and estimates  
/// - **Sales Receipts**: Point-of-sale receipts
/// - **Purchase Orders**: Vendor purchase orders
/// - **Statements**: Customer account statements
/// - **Bills**: Vendor bills (in some cases)
///
/// # Examples
///
/// ## Get PDF Bytes
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::{QBGetPDF, QBQuery}};
/// use quickbooks_types::Invoice;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Find an invoice
/// let invoice = Invoice::query_single("WHERE DocNumber = 'INV-001'", &qb_context, &client)?;
///
/// // Get PDF as bytes
/// let pdf_bytes = invoice.get_pdf_bytes(&qb_context, &client)?;
/// 
/// // Use the bytes (save to file, send via email, etc.)
/// std::fs::write("invoice_001.pdf", pdf_bytes)?;
/// ```
///
/// ## Save PDF Directly to File
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::{QBGetPDF, QBQuery}};
/// use quickbooks_types::Estimate;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Find an estimate
/// let estimate = Estimate::query_single("WHERE DocNumber = 'EST-001'", &qb_context, &client)?;
///
/// // Save directly to file
/// estimate.save_pdf_to_file("estimate_001.pdf", &qb_context, &client)?;
/// println!("PDF saved successfully!");
/// ```
///
/// ## Batch PDF Generation
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::{QBGetPDF, QBQuery}};
/// use quickbooks_types::Invoice;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Get multiple invoices
/// let invoices = Invoice::query(
///     "WHERE MetaData.CreateTime >= '2024-01-01'",
///     Some(50),
///     &qb_context,
///     &client
/// )?;
///
/// // Generate PDFs for all invoices
/// for invoice in invoices {
///     if let Some(doc_number) = &invoice.doc_number {
///         let filename = format!("invoice_{}.pdf", doc_number);
///         match invoice.save_pdf_to_file(&filename, &qb_context, &client) {
///             Ok(_) => println!("Saved {}", filename),
///             Err(e) => eprintln!("Failed to save {}: {}", filename, e),
///         }
///     }
/// }
/// ```
///
/// # PDF Content
///
/// Generated PDFs include:
/// - QuickBooks branding and company information
/// - Entity-specific formatting (invoice layout, estimate format, etc.)
/// - All line items, taxes, and totals
/// - Customer/vendor information
/// - Terms, notes, and custom fields
/// - Professional formatting suitable for printing or emailing
///
/// # File Operations
///
/// When using `save_pdf_to_file()`:
/// - File will be created or overwritten if it exists
/// - Parent directories must exist (function won't create them)
/// - File permissions depend on the operating system
/// - Large PDFs may take time to write to disk
///
/// # Return Values
///
/// - `get_pdf_bytes()`: Returns `Vec<u8>` containing the PDF binary data
/// - `save_pdf_to_file()`: Returns `()` on successful file write
///
/// # Errors
///
/// - `NoIdOnGetPDF`: Entity missing ID (must be saved to QuickBooks first)
/// - `UreqError`: Network or HTTP errors during PDF generation request
/// - `BadRequest`: QuickBooks can't generate PDF (unsupported entity, invalid data, etc.)
/// - `IoError`: File writing errors when saving to disk
/// - `ByteLengthMismatch`: Incomplete file write operation
/// - Rate limiting errors if API limits are exceeded
///
/// # Performance Notes
///
/// - PDF generation happens server-side and may take several seconds
/// - Large invoices with many line items take longer to generate  
/// - Consider caching PDFs if they'll be accessed multiple times
/// - Batch operations should include delays to avoid rate limiting
pub trait QBGetPDF {
    /// Gets the PDF bytes
    /// returns an error if the item has no ID
    /// or if the request itself fails
    fn get_pdf_bytes(&self, qb: &QBContext, client: &Agent) -> APIResult<Vec<u8>>
    where
        Self: Sized;

    /// Saves the PDF to a file
    /// returns an error if the item has no ID
    /// or if the request itself fails
    fn save_pdf_to_file(&self, file_name: &str, qb: &QBContext, client: &Agent) -> APIResult<()>
    where
        Self: Sized + QBPDFable + QBItem,
    {
        qb_save_pdf_to_file(self, file_name, qb, client)
    }
}
impl<T: QBItem + QBPDFable> QBGetPDF for T {
    fn get_pdf_bytes(&self, qb: &QBContext, client: &Agent) -> APIResult<Vec<u8>> {
        qb_get_pdf_bytes(self, qb, client)
    }
}

/// Gets the PDF bytes of the item
/// returns an error if the item has no ID
/// or if the request itself fails
fn qb_get_pdf_bytes<T: QBItem + QBPDFable>(
    item: &T,
    qb: &QBContext,
    client: &Agent,
) -> APIResult<Vec<u8>> {
    let Some(id) = item.id() else {
        return Err(APIErrorInner::NoIdOnGetPDF.into());
    };

    let request = crate::client::build_request(
        ureq::http::Method::GET,
        &format!("company/{}/{}/{}/pdf", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None::<std::iter::Empty<(&str, &str)>>,
        "application/json",
        qb.environment,
        &qb.access_token,
    )?;

    let response = qb.with_permission(|_| Ok(client.run(request)?))?;

    if !response.status().is_success() {
        return Err(APIErrorInner::BadRequest(response.into_body().read_json()?).into());
    }

    log::info!(
        "Successfully got PDF of {} with ID : {}",
        T::name(),
        item.id().ok_or(APIErrorInner::NoIdOnGetPDF)?
    );

    Ok(response.into_body().read_to_vec()?)
}

fn qb_save_pdf_to_file<T: QBItem + QBPDFable>(
    item: &T,
    file_name: &str,
    qb: &QBContext,
    client: &Agent,
) -> Result<(), APIError> {
    let bytes = qb_get_pdf_bytes(item, qb, client)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_name)?;
    let amt = file.write(&bytes)?;

    if bytes.len() != amt {
        log::error!("Couldn't write all the bytes of file : {}", file_name);
        return Err(APIErrorInner::ByteLengthMismatch.into());
    }

    log::info!(
        "Successfully saved PDF of {} #{} to {}",
        T::name(),
        item.id().ok_or(APIErrorInner::NoIdOnGetPDF)?,
        file_name
    );
    Ok(())
}
