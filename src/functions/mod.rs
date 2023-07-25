use chrono::{DateTime, Utc};
use serde::Deserialize;

pub mod create;
pub mod delete;
pub mod query;
pub mod read;

macro_rules! qb_request {
    ($qb:expr, $method:expr, $url:expr, $body:expr, $query:expr) => {{
        let request = $qb.request($method, $url, $body, $query);

        println!("{request:?}");

        let resp = $qb.http_client.execute(request).await?;

        // println!("\n{:?}", resp.text().await);

        if !resp.status().is_success() {
            return Err(APIError {
                status_code: resp.status(),
                body: resp.text().await?,
            });
        }

        resp
    }};
}

pub(crate) use qb_request;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct QBResponse<T> {
    object: T,
    time: DateTime<Utc>
}