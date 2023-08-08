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
        let id = match self.id() {
            Some(id) => id,
            None => return Err(APIError::NoIdOnSend)
        };

        let request = qb_request!(
            qb,
            reqwest::Method::POST,
            &format!("company/{}/{}/{}/send", qb.company_id, Self::qb_id(), id),
            None::<Self>,
            Some(&[("send", &format!("sendTo={email}"))])
        );

        let resp: QBResponse<Self> = request.json().await?;

        log::info!("Successfully Sent {} object : {resp:?}", Self::name());
        Ok(resp.object)
    }
}

impl<T: QBItem + QBSendable> QBSend for T {}
