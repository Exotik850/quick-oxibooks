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
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//!     invoice.save_pdf_to_file("invoice.pdf", &qb_context, &client).await?;
//!     
//!     // Alternatively, get PDF bytes
//!     let pdf_bytes = invoice.get_pdf_bytes(&qb_context, &client).await?;
//!     
//!     Ok(())
//! }
//! ```
use quickbooks_types::{QBItem, QBPDFable};
use reqwest::{Client, Method};
use tokio::io::AsyncWriteExt;

use crate::{error::APIError, Environment, QBContext};

/// Trait for getting a PDF of an item
pub trait QBGetPDF {
    /// Gets the PDF bytes
    /// returns an error if the item has no ID
    /// or if the request itself fails
    fn get_pdf_bytes(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, APIError>>
    where
        Self: Sized;

    /// Saves the PDF to a file
    /// returns an error if the item has no ID
    /// or if the request itself fails
    fn save_pdf_to_file(
        &self,
        file_name: &str,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<(), APIError>>
    where
        Self: Sized + QBPDFable + QBItem,
    {
        qb_save_pdf_to_file(self, file_name, qb, client)
    }
}
impl<T: QBItem + QBPDFable> QBGetPDF for T {
    fn get_pdf_bytes(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, APIError>> {
        qb_get_pdf_bytes(self, qb, client)
    }
}

/// Gets the PDF bytes of the item
/// returns an error if the item has no ID
/// or if the request itself fails
async fn qb_get_pdf_bytes<T: QBItem + QBPDFable>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<Vec<u8>, APIError> {
    let Some(id) = item.id() else {
        return Err(APIErrorInner::NoIdOnGetPDF);
    };

    let request = crate::client::build_request(
        Method::GET,
        &format!("company/{}/{}/{}/pdf", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        "application/json",
        qb.environment,
        client,
        &qb.access_token,
    )?;

    let response = qb.with_permission(|qb| client.execute(request)).await?;

    if !response.status().is_success() {
        return Err(APIErrorInner::BadRequest(response.json().await?));
    }

    log::debug!(
        "Successfully got PDF of {} with ID : {}",
        T::name(),
        item.id().ok_or(APIErrorInner::NoIdOnGetPDF)?
    );

    Ok(response.bytes().await?.into())
}

async fn qb_save_pdf_to_file<T: QBItem + QBPDFable>(
    item: &T,
    file_name: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<(), APIError> {
    let bytes = qb_get_pdf_bytes(item, qb, client).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_name)
        .await?;
    let amt = file.write(&bytes).await?;

    if bytes.len() != amt {
        log::error!("Couldn't write all the bytes of file : {}", file_name);
        return Err(APIErrorInner::ByteLengthMismatch);
    }

    log::debug!(
        "Successfully saved PDF of {} #{} to {}",
        T::name(),
        item.id().ok_or(APIErrorInner::NoIdOnGetPDF)?,
        file_name
    );
    Ok(())
}
