use serde::Deserialize;

#[cfg(feature = "attachments")]
pub mod attachment;
pub mod create;
pub mod delete;
#[cfg(feature = "pdf")]
pub mod pdfable;
pub mod query;
pub mod read;
pub mod send;

macro_rules! qb_request {
    ($qb:expr, $token:expr, $method:expr, $url:expr, $body:expr, $query:expr) => {{
        // Create the request
        let request = $qb.request($token, $method, $url, $body, $query).await?;

        // Send the request
        let resp = $qb.http_client.execute(request).await?;

        // Return error if the request did not go through
        if !resp.status().is_success() {
            return Err(APIError::BadRequest(resp.text().await?));
        }

        resp
    }};
}

pub(crate) use qb_request;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct QBResponse<T> {
    #[serde(
        alias = "Item",
        alias = "Account",
        alias = "Attachabe",
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
    object: T,
    time: String,
}
