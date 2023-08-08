use chrono::{DateTime, Utc};
use serde::Deserialize;

pub mod create;
pub mod delete;
pub mod query;
pub mod read;
pub mod send;

macro_rules! qb_request {
    ($qb:expr, $method:expr, $url:expr, $body:expr, $query:expr) => {{
        let request = $qb.request($method, $url, $body, $query)?;

        let resp = $qb.http_client.execute(request).await?;

        if !resp.status().is_success() {
            return Err(APIError::BadRequest(resp.text().await.unwrap()));
        }

        resp
    }};
}

pub(crate) use qb_request;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct QBResponse<T> {
    object: T,
    time: DateTime<Utc>,
}
