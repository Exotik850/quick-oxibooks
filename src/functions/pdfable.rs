use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{QBItem, QBPDFable, QBReadable};
use reqwest::Method;

use crate::client::Quickbooks;
use crate::error::APIError;

use super::{qb_request, QBResponse};

#[async_trait]
pub trait QBPDF: QBPDFable + QBItem {
    async fn get_pdf(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        if !self.can_read() {
            return Err(APIError::NoIdOnGetPDF);
        }

        let resp = qb_request!(
            qb,
            Method::POST,
            &format!("company/{}/{}", qb.company_id, Self::qb_id()),
            self,
            None
        );

        let resp: QBResponse<Self> = resp.json().await?;

        log::info!(
            "Successfully deleted {} with ID of {}",
            Self::name(),
            resp.object.id().unwrap()
        );

        Ok(resp.object)
    }
}

impl<T: QBItem + QBPDFable> QBPDF for T {}
