use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{QBDeletable, QBItem};
use reqwest::Method;
use serde::Deserialize;

use super::{qb_request, QBResponse};
use crate::client::Quickbooks;
use crate::error::APIError;

#[async_trait]
pub trait QBDelete
where
    Self: QBItem,
{
    async fn delete(&self, qb: &Quickbooks<Authorized>) -> Result<QBDeleted, APIError> {
        match (self.sync_token(), self.id()) {
            (Some(_), Some(_)) => {
                let response = qb_request!(
                    qb,
                    Method::POST,
                    &format!(
                        "company/{}/{}?operation=delete",
                        qb.company_id,
                        Self::qb_id()
                    ),
                    Some(self),
                    None
                );

                let resp: QBResponse<QBDeleted> = response.json().await?;

                log::info!(
                    "Successfully deleted {} with ID of {}",
                    Self::name(),
                    &resp.object.id
                );
                Ok(resp.object)
            }
            _ => Err(APIError::BadRequest(
                "Missing ID or Sync token on delete".into(),
            )),
        }
    }
}

impl<T: QBItem + QBDeletable> QBDelete for T {}

#[derive(Deserialize, Debug, Default)]
pub struct QBDeleted {
    pub status: String,
    pub domain: String,
    #[serde(rename = "Id")]
    pub id: String,
}
