use async_trait::async_trait;
use quickbooks_types::{QBDeletable, QBItem};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{qb_request, QBResponse};
use crate::{client::Quickbooks, error::APIError};

#[async_trait]
pub trait QBDelete
where
    Self: QBItem,
{
    async fn delete(&self, qb: &Quickbooks) -> Result<QBDeleted, APIError> {
        let (Some(_), Some(id)) = (self.sync_token(), self.id()) else {
            return Err(APIError::DeleteMissingItems);
        };

        let delete_object: QBToDelete = self.into();

        let response = qb_request!(
            qb,
            Method::POST,
            &format!(
                "company/{}/{}?operation=delete",
                qb.company_id,
                Self::qb_id()
            ),
            Some(delete_object),
            None
        );

        let resp: QBResponse<QBDeleted> = response.json().await?;

        log::info!("Successfully deleted {} with ID of {}", Self::name(), id);
        Ok(resp.object)
    }
}

impl<T: QBItem + QBDeletable> QBDelete for T {}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct QBToDelete {
    id: String,
    sync_token: String,
}

// ! For some reason TryFrom won't compile, however it is always checked if there is an ID and SyncToken before using this atm
impl<T: QBItem> From<&T> for QBToDelete {
    fn from(value: &T) -> Self {
        match (value.id().cloned(), value.sync_token().cloned()) {
            (Some(id), Some(sync_token)) => Self { id, sync_token },
            (_, _) => panic!("Couldnt delete QBItem, no ID or SyncToken available!"), // TODO Make this not possible
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct QBDeleted {
    pub status: String,
    pub domain: String,
    #[serde(rename = "Id")]
    pub id: String,
}
