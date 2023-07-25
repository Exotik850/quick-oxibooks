use async_trait::async_trait;
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;

use crate::quickbook::Quickbooks;
use crate::error::APIError;
use super::{QBResponse, qb_request};

#[async_trait]
pub trait QBRead
where Self: QBItem
{
    async fn read(&mut self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let response = qb_request!(
            qb,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), self.id().expect("Trying to read when no ID set for object")),
            (),
            None
        );

        let resp: QBResponse<Self> = response.json().await?;

        *self = resp.object.clone();
        Ok(resp.object)
    }

    async fn get(id: String, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
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