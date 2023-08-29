use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{QBItem, QBSendable};

use crate::client::Quickbooks;
use crate::error::APIError;
use crate::functions::qb_request;

use super::QBResponse;

#[async_trait]
pub trait QBSend
where
    Self: QBItem + QBSendable,
{
    async fn send_email(&self, email: &str, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        let Some(id) = self.id() else {
            return Err(APIError::NoIdOnSend);
        };

        let request = qb_request!(
            qb,
            reqwest::Method::POST,
            &format!("company/{}/{}/{}/send", qb.company_id, Self::qb_id(), id),
            None::<Self>,
            Some(&[("sendTo", email)])
        );

        let resp: QBResponse<Self> = request.json().await?;

        log::info!("Successfully Sent {} object with ID : {}", Self::name(), id);
        Ok(resp.object)
    }
}

impl<T: QBItem + QBSendable> QBSend for T {}
