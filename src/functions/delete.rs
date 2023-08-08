use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;

use super::{qb_request, QBResponse};
use crate::client::Quickbooks;
use crate::error::APIError;

#[async_trait]
pub trait QBDelete
where
    Self: QBItem,
{
    async fn delete(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
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

                // Deleting returns a diff object than normal, currently won't work
                let resp: QBResponse<Self> = response.json().await?;

                log::info!(
                    "Successfully deleted {} with ID of {}",
                    Self::name(),
                    self.id().unwrap()
                );
                Ok(resp.object)
            }
            _ => Err(APIError::BadRequest(
                "Missing ID or Sync token on delete".into(),
            )),
        }
    }
}

// TODO Not all types can be deleted, only implement trait for those that can
// impl<T: QBItem> QBDelete for T {}
