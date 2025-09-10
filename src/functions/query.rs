use quickbooks_types::QBItem;
use serde::Deserialize;
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    APIResult, QBContext,
};

use super::qb_request;

/// Trait for querying `QuickBooks` entities using SQL-like syntax.
///
/// This trait provides methods for executing `QuickBooks` queries using SQL-like syntax.
/// `QuickBooks` supports a subset of SQL including WHERE clauses, ORDER BY, and MAXRESULTS.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement [`QBItem`].
/// You don't need to implement it manually.
///
/// # Query Syntax
///
/// `QuickBooks` queries use SQL-like syntax:
/// - `WHERE field = 'value'`: Filter by field value
/// - `WHERE field IN ('val1', 'val2')`: Filter by multiple values  
/// - `WHERE field LIKE '%pattern%'`: Pattern matching
/// - `ORDER BY field ASC/DESC`: Sort results
/// - `MAXRESULTS n`: Limit number of results
///
/// # Examples
///
/// ## Basic Queries
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::query::QBQuery;
/// use quickbooks_types::{Customer, Invoice};
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
///
/// // Query active customers
/// let customers = Customer::query(
///     "WHERE Active = true ORDER BY DisplayName",
///     Some(50),
///     &qb_context,
///     &client
/// ).unwrap();
///
/// // Query recent invoices
/// let invoices = Invoice::query(
///     "WHERE TotalAmt > '1000.00' AND MetaData.CreateTime > '2024-01-01'",
///     Some(25),
///     &qb_context,
///     &client
/// ).unwrap();
/// ```
///
/// ## Single Entity Queries
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use ureq::Agent;
/// use quickbooks_types::{Customer, Invoice};
/// use quick_oxibooks::functions::query::QBQuery;
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
/// // Find a specific customer by name
/// let customer = Customer::query_single(
///     "WHERE DisplayName = 'Acme Corp'",
///     &qb_context,
///     &client
/// ).unwrap();
///
/// // Find an invoice by document number
/// let invoice = Invoice::query_single(
///     "WHERE DocNumber = 'INV-001'",
///     &qb_context,
///     &client
/// ).unwrap();
/// ```
///
/// # Field Names
///
/// Use `QuickBooks` field names (`PascalCase`) in queries, not Rust field names (`snake_case)`:
/// - Correct: `WHERE DisplayName = 'John'`
/// - Incorrect: `WHERE display_name = 'John'`
///
/// # Performance Notes
///
/// - Use `MAXRESULTS` to limit large result sets
/// - Index-friendly queries (ID, `DisplayName`) perform better
/// - Complex queries may timeout on large datasets
///
/// # Errors
///
/// - `NoQueryObjects`: No entities matched the query
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: Invalid query syntax or field names
/// - `JsonError`: Response parsing errors
pub trait QBQuery {
    /// Queries the `QuickBooks` API for objects of type T
    /// Returns a vector of objects of type T
    /// `max_results` limits the number of results returned
    /// `query_str` is the query string to use,
    ///  this will be placed into the query like so:
    /// ```ignore
    /// "select * from {type_name} {query_str} MAXRESULTS {max_results}"
    /// ```
    fn query(
        query_str: &str,
        max_results: Option<usize>,
        qb: &QBContext,
        client: &Agent,
    ) -> APIResult<Vec<Self>>
    where
        Self: Sized;

    /// Queries the `QuickBooks` API for a single object of type T
    /// Returns the object of type T
    /// `query_str` is the query string to use,
    ///  this will be placed into the query like so:
    /// ```ignore
    /// "select * from {type_name} {query_str} MAXRESULTS {max_results}"
    /// ```
    #[must_use]
    fn query_single(query_str: &str, qb: &QBContext, client: &Agent) -> APIResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::query(query_str, Some(1), qb, client)?.swap_remove(0))
    }
}

impl<T: QBItem> QBQuery for T {
    fn query(
        query_str: &str,
        max_results: Option<usize>,
        qb: &QBContext,
        client: &Agent,
    ) -> APIResult<Vec<Self>> {
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
fn qb_query<T: QBItem>(
    query_str: &str,
    max_results: Option<usize>,
    qb: &QBContext,
    client: &Agent,
) -> Result<Vec<T>, APIError> {
    let mut query = format!("select * from {} {query_str}", T::name());
    if let Some(max) = max_results {
        query.push_str(&format!(" MAXRESULTS {max}"));
    }
    let response: QueryResponseExt<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/query", qb.company_id),
        None::<&()>,
        None,
        Some([("query", query.as_str())]),
    )?;
    #[cfg(feature = "logging")]
    log::info!(
        "Successfully Queried {} {}(s) for query string : {query_str}",
        response.query_response.items.len(),
        T::name()
    );
    Ok(response.query_response.items)
}

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
