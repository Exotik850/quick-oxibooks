use async_trait::async_trait;
use quickbooks_types::QBItem;
use reqwest::Method;
use serde::Deserialize;

use super::qb_request;
use crate::{client::Quickbooks, error::APIError};

#[async_trait]
pub trait QBQuery
where
    Self: QBItem,
{
    async fn query(
        qb: &Quickbooks,
        access_token: &str,
        query_str: &str,
        max_results: usize,
    ) -> Result<Vec<Self>, APIError> {
        let response = qb_request!(
            qb,
            access_token,
            Method::GET,
            &format!("company/{}/query", qb.company_id),
            None::<Self>,
            Some(&[(
                "query",
                &format!(
                    "select * from {} {query_str} MAXRESULTS {max_results}",
                    Self::name()
                ),
            )])
        );

        let resp: QueryResponseExt<Self> = response.json().await?;

        if resp.query_response.items.is_empty() {
            log::warn!("Queried no items for query : {query_str}");
            Err(APIError::NoQueryObjects(query_str.into()))
        } else {
            log::info!(
                "Successfully Queried {} {}(s) for query string : {query_str}",
                resp.query_response.items.len(),
                Self::name()
            );
            Ok(resp.query_response.items)
        }
    }

    async fn query_single(
        qb: &Quickbooks,
        access_token: &str,
        query_str: &str,
    ) -> Result<Self, APIError> {
        Ok(Self::query(qb, access_token, query_str, 1).await?.remove(0))
    }
}

impl<T: QBItem> QBQuery for T {}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct QueryResponse<T> {
    pub total_count: i64,
    #[serde(
        alias = "Item",
        alias = "Account",
        alias = "Invoice",
        alias = "Attachable",
        alias = "Bill",
        alias = "CompanyInfo",
        alias = "Customer",
        alias = "Employee",
        alias = "Estimate",
        alias = "Payment",
        alias = "SalesReceipt",
        alias = "Vendor"
    )]
    items: Vec<T>,
    pub start_position: i64,
    pub max_results: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResponseExt<T> {
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse<T>,
    pub time: String,
}
