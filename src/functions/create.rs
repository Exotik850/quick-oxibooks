use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{QBCreatable, QBItem};
use reqwest::Method;

use crate::client::Quickbooks;
use crate::error::APIError;

use super::{qb_request, QBResponse};

#[async_trait]
pub trait QBCreate: QBCreatable + QBItem {
    async fn create(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        if !self.can_create() {
            return Err(APIError::CreateMissingItems);
        }

        let resp = qb_request!(
            qb,
            Method::POST,
            &format!("company/{}/{}", qb.company_id, Self::qb_id()),
            Some(self),
            None
        );

        let resp: QBResponse<Self> = resp.json().await?;

        log::info!(
            "Successfully created {} with ID of {}",
            Self::name(),
            resp.object.id().expect("No ID on QB object after creation")
        );

        Ok(resp.object)
    }
}

impl<T: QBItem + QBCreatable> QBCreate for T {}
