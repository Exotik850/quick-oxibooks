use chrono::{DateTime, Utc};
use intuit_oauth::Authorized;
use quickbooks_types::QBItem;
use reqwest::Method;
use serde::Deserialize;
use async_trait::async_trait;

use crate::quickbook::{Quickbooks, APIError};
use super::qb_request;

#[async_trait]
pub trait QBQuery
where Self: QBItem
{
    async fn query(qb: &Quickbooks<Authorized>, query_str: &str) -> Result<Vec<Self>, APIError> {
        let response = qb_request!(
            qb,
            Method::GET,
            &format!("company/{}/query", qb.company_id),
            (),
            Some(&[(
                "query",
                &format!(
                    "select * from {} where {query_str} MAXRESULTS 1000",
                    Self::name(),
                ),
            )])
        );

        println!("{:?}\n\n", response);

        let resp: QueryResponseExt<Self> = response.json().await?;

        Ok(resp.query_response.items)
    }
}

impl<T: QBItem> QBQuery for T {}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all="PascalCase", default)]
pub struct QueryResponse<T>
{
    pub total_count: i64,
    #[serde(default, alias="Item", alias="Account", alias="Invoice", alias="Attachable", alias="Bill",
    alias="CompanyInfo", alias="Customer", alias="Employee", alias="Estimate", alias="Payment", alias="Vendor")]
    items: Vec<T>,
    pub start_position: i64,
    pub max_results: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResponseExt<T>
{
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse<T>,
    pub time: DateTime<Utc>,
}
