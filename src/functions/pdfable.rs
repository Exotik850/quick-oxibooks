use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{QBItem, QBPDFable};
use reqwest::Method;
use tokio::io::AsyncWriteExt;

use crate::client::Quickbooks;
use crate::error::APIError;

#[async_trait]
pub trait QBPDF: QBPDFable + QBItem {
    async fn get_pdf_bytes(&self, qb: &Quickbooks<Authorized>) -> Result<Vec<u8>, APIError> {
        let id = match self.id() {
            Some(id) => id,
            None => return Err(APIError::NoIdOnGetPDF),
        };

        let path = &format!("company/{}/{}/{}/pdf", qb.company_id, Self::qb_id(), id);
        let url = qb.build_url(path, &None)?;
        let headers = qb.build_headers("application/pdf")?;
        let request = qb.build_request(&Method::GET, url, headers, &None::<Self>)?;

        let resp = qb.http_client.execute(request).await?;

        if !resp.status().is_success() {
            return Err(APIError::BadRequest(resp.text().await?))
        }

        log::info!(
            "Successfully got PDF of {} with ID : {}",
            Self::name(),
            self.id().unwrap()
        );

        Ok(resp.bytes().await?.into())
    }

    async fn save_pdf_to_file(
        &self,
        file_name: &str,
        qb: &Quickbooks<Authorized>,
    ) -> Result<(), APIError> {
        let bytes = self.get_pdf_bytes(qb).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .await?;
        file.write(&bytes).await?;
        log::info!(
            "Successfully saved {} with ID : {}",
            Self::name(),
            self.id().unwrap()
        );
        Ok(())
    }
}

impl<T: QBItem + QBPDFable> QBPDF for T {}
