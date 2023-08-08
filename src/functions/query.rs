use async_trait::async_trait;
use chrono::{DateTime, Utc};
use intuit_oxi_auth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;
use serde::Deserialize;

use super::qb_request;
use crate::client::Quickbooks;
use crate::error::APIError;

#[async_trait]
pub trait QBQuery
where
    Self: QBItem,
{
    async fn query(
        qb: &Quickbooks<Authorized>,
        query_str: &str,
        max_results: usize,
    ) -> Result<Vec<Self>, APIError> {
        let response = qb_request!(
            qb,
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

        match resp.query_response.items.is_empty() {
            false => {
                log::info!(
                    "Successfully Queried {} items for query string : {query_str}",
                    resp.query_response.items.len()
                );
                Ok(resp.query_response.items)
            }
            true => {
                log::warn!("Queried no items for query : {query_str}");
                Err(APIError::NoQueryObjects)
            }
        }
    }

    async fn query_single(qb: &Quickbooks<Authorized>, query_str: &str) -> Result<Self, APIError> {
        let mut query_results = Self::query(qb, query_str, 1).await?;
        match query_results.is_empty() {
            false => Ok(query_results.remove(0)),
            true => Err(APIError::NoQueryObjects),
        }
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
    pub time: DateTime<Utc>,
}
