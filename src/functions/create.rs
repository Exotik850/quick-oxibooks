use async_trait::async_trait;
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::{Method, StatusCode};

use crate::error::APIError;
use crate::quickbook::Quickbooks;

use super::{QBResponse, qb_request};

#[async_trait]
pub trait QBCreate
where
    Self: QBItem,
{
    async fn create(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let resp = qb_request!(
            qb,
            Method::POST,
            &format!("company/{}/{}", qb.company_id, Self::qb_id()),
            self,
            None
        );

        let resp: QBResponse<Self> = resp.json().await?;

        Ok(resp.object)
    }
}

impl<T: QBItem> QBCreate for T {}
