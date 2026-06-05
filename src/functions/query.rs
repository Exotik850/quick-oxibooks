//! Functions for querying `QuickBooks` entities using SQL-like syntax.

use std::fmt::Write;
use std::io::{BufRead, Read};

use quick_xml::events::Event;
use quick_xml::Reader;
use quickbooks_xsd::types::quick_xml::{DeserializeSync, WithDeserializer, WithSerializer};
use ureq::{http::Method, Agent};

use crate::{error::APIError, traits::QBItem, APIResult, QBContext};

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

#[derive(Debug)]
struct Empty;

impl WithSerializer for Empty {
    type Serializer<'x> = Empty;
    fn serializer<'ser>(
        &'ser self,
        _: Option<&'ser str>,
        _: bool,
    ) -> Result<Self, quickbooks_xsd::types::quick_xml::Error> {
        Ok(Empty)
    }
}

impl<'ser> quickbooks_xsd::types::quick_xml::Serializer<'ser> for Empty {
    fn next(
        &mut self,
        _: &mut quickbooks_xsd::types::quick_xml::SerializeHelper,
    ) -> Option<Result<quick_xml::events::Event<'ser>, quickbooks_xsd::types::quick_xml::Error>>
    {
        None
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
            None::<&Empty>,
            None,
            Some([("query", query_str.as_str())]),
        )
    })?;
    #[cfg(feature = "logging")]
    log::info!("Successfully Queried Raw Data for query string : {query_str}");
    let body = response.into_body().into_reader();
    parse_query_response::<T>(body)
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

/// Parse a QuickBooks query response body, extracting items of type `T`
/// from inside the `<QueryResponse>` element.
///
/// Expected structure:
/// ```xml
/// <IntuitResponse xmlns="..." time="...">
///     <QueryResponse startPosition="1" maxResults="N" totalCount="N">
///         <EntityName>...</EntityName>
///     </QueryResponse>
/// </IntuitResponse>
/// ```
fn parse_query_response<T: WithDeserializer>(reader: impl Read) -> Result<Vec<T>, APIError> {
    let mut reader = Reader::from_reader(std::io::BufReader::new(reader));
    reader.config_mut().trim_text(true);

    let mut outer_buf = Vec::new();
    let mut inner_buf = Vec::new();
    let mut raw_buf = Vec::new();
    let mut items = Vec::new();

    // Navigate to <QueryResponse>
    loop {
        match reader.read_event_into(&mut outer_buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"QueryResponse" => break,
            Event::Eof => return Ok(items),
            _ => {}
        }
        outer_buf.clear();
    }
    outer_buf.clear();

    // Read each direct child of <QueryResponse>, reconstruct its XML, deserialize
    loop {
        match reader.read_event_into(&mut outer_buf)? {
            Event::Start(ref e) => {
                raw_buf.clear();
                read_element_body_into(&mut reader, e, &mut inner_buf, &mut raw_buf)?;
                items.push(deserialize_from_bytes::<T>(&raw_buf)?);
            }
            Event::Empty(ref e) => {
                raw_buf.clear();
                write_self_closing_tag_to(&mut raw_buf, e);
                items.push(deserialize_from_bytes::<T>(&raw_buf)?);
            }
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        outer_buf.clear();
    }

    Ok(items)
}

/// Read events for one element body and reconstruct its XML bytes into `raw`.
///
/// `buf` is a reusable event buffer (distinct from the outer loop's buffer,
/// so the `BytesStart` borrow from the caller doesn't conflict).
fn read_element_body_into(
    reader: &mut Reader<impl BufRead>,
    start: &quick_xml::events::BytesStart,
    buf: &mut Vec<u8>,
    raw: &mut Vec<u8>,
) -> Result<(), quick_xml::Error> {
    write_tag_start_to(raw, start);
    let mut depth: u32 = 1;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(ref e) => {
                depth += 1;
                write_tag_start_to(raw, e);
            }
            Event::End(ref e) => {
                depth -= 1;
                raw.extend_from_slice(b"</");
                raw.extend_from_slice(e.name().as_ref());
                raw.extend_from_slice(b">");
                if depth == 0 {
                    break;
                }
            }
            Event::Empty(ref e) => write_self_closing_tag_to(raw, e),
            Event::Text(ref e) => raw.extend_from_slice(e.as_ref()),
            Event::CData(ref e) => {
                raw.extend_from_slice(b"<![CDATA[");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b"]]>");
            }
            Event::Comment(ref e) => {
                raw.extend_from_slice(b"<!--");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b"-->");
            }
            Event::Decl(ref e) => {
                raw.extend_from_slice(b"<?");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b"?>");
            }
            Event::PI(ref e) => {
                raw.extend_from_slice(b"<?");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b"?>");
            }
            Event::DocType(ref e) => {
                raw.extend_from_slice(b"<!DOCTYPE ");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b">");
            }
            Event::GeneralRef(ref e) => {
                raw.extend_from_slice(b"&");
                raw.extend_from_slice(e.as_ref());
                raw.extend_from_slice(b";");
            }
            Event::Eof => break,
        }
        buf.clear();
    }

    Ok(())
}

/// Append a start-tag to `raw`: `<Name attr="val">`.
fn write_tag_start_to(raw: &mut Vec<u8>, e: &quick_xml::events::BytesStart) {
    raw.extend_from_slice(b"<");
    raw.extend_from_slice(e.name().as_ref());
    for attr in e.attributes().flatten() {
        raw.extend_from_slice(b" ");
        raw.extend_from_slice(attr.key.as_ref());
        raw.extend_from_slice(b"=\"");
        raw.extend_from_slice(&attr.value);
        raw.extend_from_slice(b"\"");
    }
    raw.extend_from_slice(b">");
}

/// Append a self-closing tag to `raw`: `<Name attr="val"/>`.
fn write_self_closing_tag_to(raw: &mut Vec<u8>, e: &quick_xml::events::BytesStart) {
    raw.extend_from_slice(b"<");
    raw.extend_from_slice(e.name().as_ref());
    for attr in e.attributes().flatten() {
        raw.extend_from_slice(b" ");
        raw.extend_from_slice(attr.key.as_ref());
        raw.extend_from_slice(b"=\"");
        raw.extend_from_slice(&attr.value);
        raw.extend_from_slice(b"\"");
    }
    raw.extend_from_slice(b"/>");
}

/// Deserialize raw XML bytes into `T` via its `WithDeserializer`.
fn deserialize_from_bytes<T: WithDeserializer>(bytes: &[u8]) -> Result<T, APIError> {
    let mut reader = quickbooks_xsd::types::quick_xml::IoReader::new(bytes);
    T::deserialize(&mut reader).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::types::prelude::Invoice;

    #[test]
    fn test_query_response_deserializer() {
        let xml = include_str!("../../test/data/query_response.xml");
        let items: Vec<Invoice> = super::parse_query_response(xml.as_bytes()).unwrap();
        assert_eq!(items.len(), 1);
        let inv = &items[0];
        assert_eq!(inv.id(), Some(&"239".to_string()));
        assert_eq!(inv.doc_number(), Some(&"1070".to_string()));
        assert_eq!(inv.total_amt(), Some(&150.00));
    }

    #[test]
    fn test_query_response_deserializer_empty() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
                              <QueryResponse/>
                          </IntuitResponse>"#;
        let items: Vec<Invoice> = super::parse_query_response(xml.as_bytes()).unwrap();
        assert!(items.is_empty());
    }
}
