use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;

use super::{qb_request, QBResponse};
use crate::client::Quickbooks;
use crate::error::APIError;

#[async_trait]
pub trait QBRead
where
    Self: QBItem,
{
    async fn read(&mut self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let id = match self.id() {
            Some(id) => id,
            None => return Err(APIError::NoIdOnRead),
        };

        let response = qb_request!(
            qb,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), id),
            (),
            None
        );

        let resp: QBResponse<Self> = response.json().await?;

        log::info!("Successfully Read {} object : {resp:?}", Self::name());
        *self = resp.object.clone();
        Ok(resp.object)
    }

    async fn get(id: &str, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let response = qb_request!(
            qb,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), id),
            (),
            None
        );
        let resp: QBResponse<Self> = response.json().await?;
        Ok(resp.object)
    }
}

impl<T: QBItem> QBRead for T {}
