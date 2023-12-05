/*
 * A rust library for interacting with the QuickBooks API.
 *
 * For more information, you can check out their documentation at:
 * https://developer.intuit.com/app/developer/qbo/docs/develop
 *
 * ORIGINIALLY FROM https://github.com/oxidecomputer/cio
 * LICENSED UNDER APACHE 2.0
 *
 */
#![warn(clippy::pedantic)]

pub mod batch;
pub mod client;
pub mod error;

pub mod types {
    pub use quickbooks_types::*;
}

mod functions;
pub mod actions {
    pub use crate::functions::{
        create::QBCreate, delete::QBDelete, query::QBQuery, read::QBRead, send::QBSend
    };
}

pub use intuit_oxi_auth::{Authorized, Environment, Unauthorized};

#[cfg(feature = "attachments")]
pub use crate::functions::attachment::QBAttachment;
#[cfg(feature = "pdf")]
pub use crate::functions::pdfable::QBPDF;

#[cfg(feature = "macros")]
pub mod macros;
