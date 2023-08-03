/*!
 * A rust library for interacting with the QuickBooks API.
 *
 * For more information, you can check out their documentation at:
 * https://developer.intuit.com/app/developer/qbo/docs/develop
 *
 * ORIGINIALLY FROM https://github.com/oxidecomputer/cio
 * LICENSED UNDER APACHE 2.0
 *
 */

pub mod client;
pub mod error;

pub mod types {
    pub use quickbooks_types::*;
}

mod functions;
pub mod actions {
    pub use crate::functions::create::QBCreate;
    pub use crate::functions::delete::QBDelete;
    pub use crate::functions::query::QBQuery;
    pub use crate::functions::read::QBRead;
}

pub use intuit_oxi_auth::{Authorized, Environment, Unauthorized};

#[cfg(feature = "cache")]
pub use intuit_oxi_auth::Cache;
