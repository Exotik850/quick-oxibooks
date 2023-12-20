use async_trait::async_trait;
use quickbooks_types::QBItem;
use reqwest::Method;

use super::{qb_request, QBResponse};
use crate::{client::Quickbooks, error::APIError};

#[async_trait]
pub trait QBRead
where
    Self: QBItem,
{
    async fn read(&mut self, qb: &Quickbooks, access_token: &str) -> Result<(), APIError> {
        let Some(id) = self.id() else {
            return Err(APIError::NoIdOnRead);
        };

        let response = qb_request!(
            qb,
            access_token,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), id),
            None::<Self>,
            None
        );

        let resp: QBResponse<Self> = response.json().await?;

        log::info!(
            "Successfully Read {} object with ID : {}",
            Self::name(),
            resp.object.id().expect("No ID after reading QB Object")
        );

        *self = resp.object;

        Ok(())
    }

    async fn get(id: &str, qb: &Quickbooks, access_token: &str) -> Result<Self, APIError> {
        let response = qb_request!(
            qb,
            access_token,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), id),
            None::<Self>,
            None
        );
        let resp: QBResponse<Self> = response.json().await?;
        Ok(resp.object)
    }
}

impl<T: QBItem> QBRead for T {}
