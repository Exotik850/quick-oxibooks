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

pub mod client;
pub mod error;

pub mod types {
    pub use quickbooks_types::*;
}

mod functions;
pub mod actions {
    pub use crate::functions::attachment::QBAttachment;
    pub use crate::functions::create::QBCreate;
    pub use crate::functions::delete::QBDelete;
    pub use crate::functions::pdfable::QBPDF;
    pub use crate::functions::query::QBQuery;
    pub use crate::functions::read::QBRead;
    pub use crate::functions::send::QBSend;
}

pub use intuit_oxi_auth::{Authorized, Environment, Unauthorized};

#[cfg(feature = "macros")]
pub mod macros;
