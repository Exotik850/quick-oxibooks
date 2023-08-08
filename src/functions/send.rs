use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::QBItem;

use crate::client::Quickbooks;
use crate::error::APIError;
use crate::functions::qb_request;

use super::QBResponse;

#[async_trait]
pub trait QBCreate
where
    Self: QBItem,
{
    async fn create(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let request = qb_request!(
            qb,
            reqwest::Method::POST,
            &format!("company/{}/{}", qb.company_id, Self::qb_id()),
            self,
            None
        );

        let resp: QBResponse<Self> = request.json().await?;

        log::info!("Successfully Created {} object : {resp:?}", Self::name());
        Ok(resp.object)
    }
}

impl<T: QBItem> QBCreate for T {}
