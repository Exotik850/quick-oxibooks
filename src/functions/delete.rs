use async_trait::async_trait;
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;

use super::{qb_request, QBResponse};
use crate::error::APIError;
use crate::quickbook::Quickbooks;

#[async_trait]
pub trait QBDelete
where
    Self: QBItem,
{
    async fn delete(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let response = qb_request!(
            qb,
            Method::POST,
            &format!(
                "company/{}/{}?operation=delete",
                qb.company_id,
                Self::qb_id()
            ),
            self,
            None
        );

        // Deleting returns a diff object than normal, currently won't work
        let resp: QBResponse<Self> = response.json().await?;

        Ok(resp.object)
    }
}

impl<T: QBItem> QBDelete for T {}
