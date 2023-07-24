use chrono::{DateTime, Utc};
use quickbooks_types::QBItem;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryResponse<T>
    where T: QBItem
{
    #[serde(default, rename = "totalCount")]
    pub total_count: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Item")]
    items: Vec<T>,
    #[serde(default, rename = "startPosition")]
    pub start_position: i64,
    #[serde(default, rename = "maxResults")]
    pub max_results: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponseExt<T>
where T: QBItem,
{
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse<T>,
    pub time: DateTime<Utc>,
}
