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

/// Trait for getting a PDF of an item
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
