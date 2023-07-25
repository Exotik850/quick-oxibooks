use async_trait::async_trait;
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::{Method, StatusCode};

use super::{qb_request, QBResponse};
use crate::error::APIError;
use crate::quickbook::Quickbooks;

#[async_trait]
pub trait QBRead
where
    Self: QBItem,
{
    async fn read(&mut self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let id = match self.id() {
            Some(id) => id,
            None => {
                return Err(APIError {
                    status_code: StatusCode::NOT_FOUND,
                    body: "No Id set for object when trying to grab from QB".to_owned(),
                })
            }
        };

        let response = qb_request!(
            qb,
            Method::GET,
            &format!("company/{}/{}/{}", qb.company_id, Self::qb_id(), id),
            (),
            None
        );

        let resp: QBResponse<Self> = response.json().await?;

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
