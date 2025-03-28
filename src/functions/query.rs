use http_client::{http_types::Method, HttpClient};
use quickbooks_types::QBItem;
use serde::Deserialize;

use super::qb_request;
use crate::{error::APIError, QBContext};

/// Trait for querying `QuickBooks` objects
pub trait QBQuery {
    /// Queries the `QuickBooks` API for objects of type T
    /// Returns a vector of objects of type T
    /// `max_results` limits the number of results returned
    /// `query_str` is the query string to use,
    ///  this will be placed into the query like so:
    /// ```ignore
    /// "select * from {type_name} {query_str} MAXRESULTS {max_results}"
    /// ```
    fn query<Client>(
        query_str: &str,
        max_results: usize,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, APIError>>
    where
        Self: Sized,
        Client: HttpClient;

    /// Queries the `QuickBooks` API for a single object of type T
    /// Returns the object of type T
    /// `query_str` is the query string to use,
    ///  this will be placed into the query like so:
    /// ```ignore
    /// "select * from {type_name} {query_str} MAXRESULTS {max_results}"
    /// ```
    #[must_use]
    fn query_single<Client>(
        query_str: &str,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Self, APIError>>
    where
        Self: Sized,
        Client: HttpClient,
    {
        async { Ok(Self::query(query_str, 1, qb, client).await?.swap_remove(0)) }
    }
}

impl<T: QBItem> QBQuery for T {
    fn query<Client>(
        query_str: &str,
        max_results: usize,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, APIError>>
    where
        Client: HttpClient,
    {
        qb_query(query_str, max_results, qb, client)
    }
}

/// Query the quickbooks context using the query string,
/// The type determines what type of quickbooks object you are
/// Query `QuickBooks` for objects matching the query string
///
/// Builds a query using the `query_str` and queries for objects of
/// type `T`. Returns up to `max_results` objects in a `Vec`.
///
/// The `query_str` parameter will be placed into the query
/// like so:
/// ```ignore
///  "select * from {type_name} {query_str} MAXRESULTS {max_results}"
/// ```
async fn qb_query<T, Client>(
    query_str: &str,
    max_results: usize,
    qb: &QBContext,
    client: &Client,
) -> Result<Vec<T>, APIError>
where
    T: QBItem,
    Client: HttpClient,
{
    let response: QueryResponseExt<T> = qb_request(
        qb,
        client,
        Method::Get,
        &format!("company/{}/query", qb.company_id),
        None::<&()>,
        None,
        Some(&[(
            "query",
            &format!(
                "select * from {} {query_str} MAXRESULTS {max_results}",
                T::name()
            ),
        )]),
    )
    .await?;

    if response.query_response.items.is_empty() {
        log::warn!("Queried no items for query : {query_str}");
        Err(APIError::NoQueryObjects(query_str.into()))
    } else {
        log::info!(
            "Successfully Queried {} {}(s) for query string : {query_str}",
            response.query_response.items.len(),
            T::name()
        );
        Ok(response.query_response.items)
    }
}

// /// Gets a single object via query from the `QuickBooks` API
// ///
// /// Handles retrieving a `QBItem` via query,
// /// refer to `qb_query` for more details
// async fn qb_query_single<T: QBItem>(
//     query_str: &str,
//     qb: &QBContext,
//     client: &Client,
// ) -> Result<T, APIError> {
//     Ok(qb_query(query_str, 1, qb, client).await?.swap_remove(0))
// }

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
struct QueryResponse<T> {
    total_count: i64,
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
    start_position: i64,
    max_results: i64,
}

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone, Deserialize)]
struct QueryResponseExt<T> {
    #[serde(default, rename = "QueryResponse")]
    query_response: QueryResponse<T>,
    #[allow(dead_code)]
    time: String,
}
