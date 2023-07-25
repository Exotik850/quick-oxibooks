use async_trait::async_trait;
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::{Method, StatusCode};

use crate::quickbook::{APIError, Quickbooks};

use super::QBResponse;

#[async_trait]
pub trait QBCreate
where Self: QBItem
{
    async fn create(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let request = qb.request(
            Method::POST,
            &format!("company/{}/{}", qb.company_id, Self::qb_id()),
            self,
            None,
        );

        let resp = qb.http_client.execute(request).await?;
        match resp.status() {
            StatusCode::OK => (),
            s => {
                return Err(APIError {
                    status_code: s,
                    body: resp.text().await?,
                })
            }
        };

        let resp: QBResponse<Self> = resp.json().await?;

        Ok(resp.object)
    }
}

impl<T: QBItem> QBCreate for T {}