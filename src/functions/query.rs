//! Functions for querying `QuickBooks` entities using SQL-like syntax.

use std::fmt::Write;

use quickbooks_xsd::types::quick_xml::{
    DeserializeHelper, Deserializer, DeserializerArtifact, DeserializerEvent, DeserializerOutput,
    DeserializerResult, Event, WithDeserializer,
};
use ureq::{http::Method, Agent};

use crate::{error::APIError, functions::Empty, traits::QBItem, APIResult, QBContext};

use super::execute_request;

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
    fn query_single(query_str: &str, qb: &QBContext, client: &Agent) -> APIResult<Option<Self>>
    where
        Self: Sized,
    {
        let mut results = Self::query(query_str, Some(1), qb, client)?;
        if results.is_empty() {
            return Ok(None);
        }
        Ok(Some(results.swap_remove(0)))
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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Execute an arbitrary query string against the QuickBooks API.
///
/// # Safety
///
/// The caller must guarantee the query string is well-formed and safe.
/// Malformed queries may result in undefined behaviour or errors from the API.
pub unsafe fn qb_query_raw<T: QBItem>(
    query_str: impl std::fmt::Display,
    qb: &QBContext,
    client: &Agent,
) -> Result<Vec<T>, APIError> {
    let query_str = format!("{query_str}");
    let response = qb.with_permission(|qb| {
        execute_request::<&str, &str, Empty>(
            qb,
            client,
            Method::GET,
            &format!("company/{}/query", qb.company_id),
            None,
            None,
            Some([("query", query_str.as_str())]),
        )
    })?;
    #[cfg(feature = "logging")]
    log::info!("Successfully Queried Raw Data for query string : {query_str}");

    let body = response.into_body().into_reader();
    let wrapper: QueryResponseItems<T> = crate::xml_deserialize(body)?;
    Ok(wrapper.0)
}

/// Build and execute a `select * from {type} {query_str}` query.
fn qb_query<T: QBItem>(
    query_str: &str,
    max_results: Option<usize>,
    qb: &QBContext,
    client: &Agent,
) -> Result<Vec<T>, APIError> {
    let mut query = format!("select * from {} {query_str}", T::NAME);
    if let Some(max) = max_results {
        write!(&mut query, " MAXRESULTS {max}").expect("Writing to string should not fail");
    }
    unsafe { qb_query_raw::<T>(query, qb, client) }
}

// ---------------------------------------------------------------------------
// QueryResponseItems<T> — deserializes a <QueryResponse> containing Vec<T>
// ---------------------------------------------------------------------------

/// Wrapper type whose [`Deserializer`] collects items of `T` from inside a
/// `<QueryResponse>` element.
#[derive(Debug)]
struct QueryResponseItems<T>(Vec<T>);

impl<T: QBItem> WithDeserializer for QueryResponseItems<T> {
    type Deserializer = QueryResponseItemsDe<T>;
}

// ---------------------------------------------------------------------------
// Deserializer state machine
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct QueryResponseItemsDe<T: QBItem> {
    /// Accumulated items.
    items: Vec<T>,
    /// When `Some`, we are inside a child and delegating.
    child: Option<T::Deserializer>,
    /// `true` once we've seen `<QueryResponse>`.
    inside_response: bool,
}

impl<T: QBItem> QueryResponseItemsDe<T> {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            child: None,
            inside_response: false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct QBDeError(pub String);

impl std::fmt::Display for QBDeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryResponseItemsDe error: {}", self.0)
    }
}
impl std::error::Error for QBDeError {}

impl<'de, T: QBItem> Deserializer<'de, QueryResponseItems<T>> for QueryResponseItemsDe<T> {
    fn init(
        _helper: &mut DeserializeHelper,
        _event: Event<'de>,
    ) -> DeserializerResult<'de, QueryResponseItems<T>> {
        // The first event is the outer <IntuitResponse> Start — silently
        // consume it. Subsequent events flow through `next`.
        Ok(DeserializerOutput {
            artifact: DeserializerArtifact::Deserializer(Self::new()),
            event: DeserializerEvent::None,
            allow_any: false,
        })
    }

    fn next(
        mut self,
        helper: &mut DeserializeHelper,
        event: Event<'de>,
    ) -> DeserializerResult<'de, QueryResponseItems<T>> {
        // ── Delegate to active child deserializer ────────────────────
        if let Some(child) = self.child.take() {
            let output = child.next(helper, event)?;

            let (data, next_child) = output.artifact.into_parts();
            if let Some(item) = data {
                self.items.push(item);
            } else if next_child.is_none() {
                return Err(quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
                    "Child deserializer finished without producing an item".to_string(),
                )));
            }
            self.child = next_child;

            // Child consumed the event — keep waiting.
            return Ok(DeserializerOutput {
                artifact: DeserializerArtifact::Deserializer(self),
                event: DeserializerEvent::None,
                allow_any: false,
            });
        }

        // ── Not inside a child — handle the event ourselves ──────────
        if !self.inside_response {
            // Searching for <QueryResponse>
            if matches!(&event, Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"QueryResponse")
            {
                self.inside_response = true;
                let artifact = if matches!(event, Event::Empty(_)) {
                    DeserializerArtifact::Data(QueryResponseItems(self.items))
                } else {
                    DeserializerArtifact::Deserializer(self)
                };
                return Ok(DeserializerOutput {
                    artifact,
                    event: DeserializerEvent::None,
                    allow_any: true,
                });
            }
            // Skip anything before <QueryResponse>.
            let allow_any = matches!(event, Event::Start(_) | Event::Empty(_));
            return Ok(DeserializerOutput {
                artifact: DeserializerArtifact::Deserializer(self),
                event: DeserializerEvent::None,
                allow_any,
            });
        }

        // Inside <QueryResponse>
        match &event {
            Event::Start(_) | Event::Empty(_) => {
                // Only delegate to the child deserializer when the element
                // name matches the expected entity. Any other element is
                // unexpected inside <QueryResponse> and produces an error.
                let name = match &event {
                    Event::Start(e) | Event::Empty(e) => e.name(),
                    _ => unreachable!(),
                };
                if name.as_ref() != T::NAME.as_bytes() {
                    return Err(quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
                        format!(
                            "Unexpected element <{}> inside <QueryResponse>, expected <{}>",
                            String::from_utf8_lossy(name.as_ref()),
                            T::NAME
                        ),
                    )));
                }

                let child_output = T::init(helper, event)?;
                let (item, child_de) = child_output.artifact.into_parts();
                if let Some(item) = item {
                    self.items.push(item);
                }
                self.child = child_de;
                return Ok(DeserializerOutput {
                    artifact: DeserializerArtifact::Deserializer(self),
                    event: DeserializerEvent::None,
                    allow_any: false,
                });
            }
            Event::End(e) if e.name().as_ref() == b"QueryResponse" => {
                return Ok(DeserializerOutput {
                    artifact: DeserializerArtifact::Data(QueryResponseItems(self.items)),
                    event: DeserializerEvent::None,
                    allow_any: false,
                });
            }
            _ =>
            // Skip any non-start events inside <QueryResponse>.
            {
                return Ok(DeserializerOutput {
                    artifact: DeserializerArtifact::Deserializer(self),
                    event: DeserializerEvent::None,
                    allow_any: false,
                });
                // return Err(quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
                //     format!("Unexpected event inside <QueryResponse>: {event:?}"),
                // )));
            }
        }
    }

    fn finish(
        self,
        _helper: &mut DeserializeHelper,
    ) -> Result<QueryResponseItems<T>, quickbooks_xsd::types::quick_xml::Error> {
        Ok(QueryResponseItems(self.items))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::prelude::{Customer, Invoice};

    #[test]
    fn test_invoice_deserializer() {
        let xml = r#"<Invoice domain="QBO" sparse="false">
            <Id>239</Id>
            <SyncToken>0</SyncToken>
            <DocNumber>1070</DocNumber>
        </Invoice>"#;
        use quickbooks_xsd::types::quick_xml::{DeserializeSync, IoReader};
        let mut reader = IoReader::new(std::io::BufReader::new(xml.as_bytes()));
        let _inv: Invoice = Invoice::deserialize(&mut reader).unwrap();
    }

    #[test]
    fn test_query_response_with_simple_type() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
            <QueryResponse>
                <Customer domain="QBO" sparse="false">
                    <Id>123</Id>
                    <SyncToken>4</SyncToken>
                    <DisplayName>Test Customer</DisplayName>
                </Customer>
            </QueryResponse>
        </IntuitResponse>"#;
        use super::QueryResponseItems;
        let items: QueryResponseItems<Customer> = crate::xml_deserialize(xml.as_bytes()).unwrap();
        assert_eq!(items.0.len(), 1);
        assert_eq!(items.0[0].id(), Some(&"123".to_string()));
    }

    #[test]
    fn test_query_response_deserializer() {
        let xml = include_str!("../../test/data/query_response.xml");
        use super::QueryResponseItems;
        let items: QueryResponseItems<Invoice> = crate::xml_deserialize(xml.as_bytes()).unwrap();
        assert_eq!(items.0.len(), 1);
        let inv = &items.0[0];
        assert_eq!(inv.id(), Some(&"239".to_string()));
        assert_eq!(inv.doc_number(), Some(&"1070".to_string()));
        assert_eq!(inv.total_amt(), Some(&150.00));
    }

    #[test]
    fn test_query_response_deserializer_empty() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
                              <QueryResponse/>
                          </IntuitResponse>"#;
        use super::QueryResponseItems;
        let items: QueryResponseItems<Invoice> = crate::xml_deserialize(xml.as_bytes()).unwrap();
        assert!(items.0.is_empty());
    }

    #[test]
    fn test_invalid_query_response() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
                              <QueryResponse>
                                  <InvalidElement/>
                              </QueryResponse>
                          </IntuitResponse>"#;
        use super::QueryResponseItems;
        let result: Result<QueryResponseItems<Invoice>, _> = crate::xml_deserialize(xml.as_bytes());
        dbg!(&result);
        assert!(result.is_err());
    }
}
