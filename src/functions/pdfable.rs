use async_trait::async_trait;
use quickbooks_types::{QBItem, QBPDFable};
use reqwest::Method;
use tokio::io::AsyncWriteExt;

use crate::{client::Quickbooks, error::APIError};

#[async_trait]
pub trait QBPDF: QBPDFable + QBItem {
    async fn get_pdf_bytes(
        &self,
        qb: &Quickbooks,
        access_token: &str,
    ) -> Result<Vec<u8>, APIError> {
        let Some(id) = self.id() else {
            return Err(APIError::NoIdOnGetPDF);
        };

        let path = &format!("company/{}/{}/{}/pdf", qb.company_id, Self::qb_id(), id);
        let url = qb.build_url(path, None)?;
        let headers = qb.build_headers("application/pdf", access_token)?;
        let request = qb.build_request(&Method::GET, url, headers, &None::<Self>)?;

        let resp = qb.http_client.execute(request).await?;

        if !resp.status().is_success() {
            return Err(APIError::BadRequest(resp.text().await?));
        }

        log::info!(
            "Successfully got PDF of {} with ID : {}",
            Self::name(),
            self.id().ok_or(APIError::NoIdOnGetPDF)?
        );

        Ok(resp.bytes().await?.into())
    }

    async fn save_pdf_to_file(
        &self,
        file_name: &str,
        qb: &Quickbooks,
        access_token: &str,
    ) -> Result<(), APIError> {
        let bytes = self.get_pdf_bytes(qb, access_token).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .await?;
        let amt = file.write(&bytes).await?;

        if bytes.len() != amt {
            log::error!("Couldn't write all the bytes of file : {}", file_name);
            return Err(APIError::ByteLengthMismatch);
        }

        log::info!(
            "Successfully saved PDF of {} #{} to {}",
            Self::name(),
            self.id().ok_or(APIError::NoIdOnGetPDF)?,
            file_name
        );
        Ok(())
    }
}

impl<T: QBItem + QBPDFable> QBPDF for T {}
