//! Functions for reading individual `QuickBooks` entities by ID.

use std::fmt::Write;

use quickbooks_xsd::types::quick_xml::{
    DeserializeHelper, Deserializer, DeserializerArtifact, DeserializerEvent, DeserializerOutput,
    DeserializerResult, Event, SerializeHelper, Serializer, WithDeserializer, WithSerializer,
};
use ureq::{http::Method, Agent};

use crate::{error::APIError, traits::QBItem, APIResult, QBContext};

use super::execute_request;

/// Trait for reading individual `QuickBooks` entities by ID.
///
/// This trait provides methods for reading a single `QuickBooks` entity by its ID.
/// `QuickBooks` supports reading entities via the `/entity/{id}` endpoint.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement [`QBItem`].
/// You don't need to implement it manually.
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::read::QBRead;
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
/// // Read a specific customer by ID
/// let customer = Customer::read("123", &qb_context, &client).unwrap();
///
/// // Read an invoice by ID
/// let invoice = Invoice::read("456", &qb_context, &client).unwrap();
/// ```
///
/// # Errors
///
/// - `NoQueryObjects`: No entity matched the ID
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: Invalid ID
/// - `JsonError`: Response parsing errors
pub trait QBRead {
    /// Reads a single `QuickBooks` entity by ID.
    /// Returns the entity of type T
    /// `id` is the ID of the entity to read
    fn read(id: &str, qb: &QBContext, client: &Agent) -> APIResult<Self>
    where
        Self: Sized;
}

impl<T: QBItem> QBRead for T {
    fn read(id: &str, qb: &QBContext, client: &Agent) -> APIResult<Self> {
        qb_read(id, qb, client)
    }
}

// ---------------------------------------------------------------------------
// Minimal empty type for serialization (used when there is no request body)
// ---------------------------------------------------------------------------

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

impl<'ser> Serializer<'ser> for Empty {
    fn next(
        &mut self,
        _: &mut SerializeHelper,
    ) -> Option<Result<Event<'ser>, quickbooks_xsd::types::quick_xml::Error>> {
        None
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Execute an arbitrary read request against the QuickBooks API for a single entity.
///
/// # Safety
///
/// The caller must guarantee the ID is valid and safe.
/// Malformed IDs may result in undefined behaviour or errors from the API.
pub unsafe fn qb_read_raw<T: QBItem>(
    id: impl std::fmt::Display,
    qb: &QBContext,
    client: &Agent,
) -> Result<T, APIError> {
    let id_str = format!("{id}");
    let response = qb.with_permission(|qb| {
        execute_request::<&str, &str, Empty>(
            qb,
            client,
            Method::GET,
            &format!("company/{}/{}", qb.company_id, id_str),
            None::<&Empty>,
            None,
            None::<std::iter::Empty<(&str, &str)>>,
        )
    })?;
    #[cfg(feature = "logging")]
    log::info!("Successfully Read Raw Data for ID : {id_str}");

    let body = response.into_body().into_reader();
    let wrapper: QBResponse<T> = crate::xml_deserialize(body)?;
    Ok(wrapper.0)
}

/// Build and execute a read request for a single entity.
fn qb_read<T: QBItem>(id: &str, qb: &QBContext, client: &Agent) -> Result<T, APIError> {
    unsafe { qb_read_raw::<T>(id, qb, client) }
}

// ---------------------------------------------------------------------------
// QBResponse<T> — deserializes a single entity from <IntuitResponse>
// ---------------------------------------------------------------------------

/// Wrapper type whose [`Deserializer`] collects a single item of `T` from inside an
/// `<IntuitResponse>` element (used for read operations).
#[derive(Debug)]
pub struct QBResponse<T>(pub T);

impl<T: QBItem> WithDeserializer for QBResponse<T> {
    type Deserializer = QBResponseDe<T>;
}

// ---------------------------------------------------------------------------
// Deserializer state machine
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct QBResponseDe<T: QBItem> {
    /// The deserialized item.
    item: Option<T>,
    /// When `Some`, we are inside the entity and delegating.
    child: Option<T::Deserializer>,
    /// `true` once we've seen `<IntuitResponse>`.
    inside_intuit_response: bool,
    /// `true` once we've seen the start of the entity.
    seen_entity_start: bool,
}

#[derive(Debug)]
pub(crate) struct QBDeError(pub String);

impl std::fmt::Display for QBDeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QBResponseDe error: {}", self.0)
    }
}
impl std::error::Error for QBDeError {}

impl<'de, T: QBItem> Deserializer<'de, QBResponse<T>> for QBResponseDe<T> {
    fn init(
        _helper: &mut DeserializeHelper,
        event: Event<'de>,
    ) -> DeserializerResult<'de, QBResponse<T>> {
        // The first event is the outer <IntuitResponse> Start — silently
        // consume it. We are now inside <IntuitResponse>, so mark it as such
        // and wait for the entity's start element to arrive in `next`.
        //
        // An empty <IntuitResponse/> is invalid for a read operation, which
        // always expects exactly one entity.
        if matches!(event, Event::Empty(_)) {
            return Err(quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
                "Empty IntuitResponse encountered".to_string(),
            )));
        }
        Ok(DeserializerOutput {
            artifact: DeserializerArtifact::Deserializer(Self {
                item: None,
                child: None,
                inside_intuit_response: true,
                seen_entity_start: false,
            }),
            event: DeserializerEvent::None,
            allow_any: false,
        })
    }

    fn next(
        mut self,
        helper: &mut DeserializeHelper,
        event: Event<'de>,
    ) -> DeserializerResult<'de, QBResponse<T>> {
        loop {
            // ── Delegate to active child deserializer ────────────────────
            if let Some(child) = self.child.take() {
                let output = child.next(helper, event)?;

                let (data, next_child) = output.artifact.into_parts();
                if let Some(item) = data {
                    self.item = Some(item);
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
            // `init` already consumed the outer <IntuitResponse> Start, so we
            // are always inside it here.
            debug_assert!(self.inside_intuit_response);

            match &event {
                Event::Start(_) | Event::Empty(_) => {
                    if self.seen_entity_start {
                        // We already saw the entity start, so this is an
                        // unexpected element. Skip it (and its subtree for a
                        // Start) by letting the orchestrator handle it.
                        return Ok(DeserializerOutput {
                            artifact: DeserializerArtifact::Deserializer(self),
                            event: DeserializerEvent::None,
                            allow_any: true,
                        });
                    }

                    // Get the local name of the event.
                    let name = match &event {
                        Event::Start(e) | Event::Empty(e) => e.name(),
                        _ => unreachable!(),
                    };
                    if name.as_ref() == T::NAME.as_bytes() {
                        // This is the entity we want.
                        self.seen_entity_start = true;
                        let child_output = T::init(helper, event)?;
                        let (item, child_de) = child_output.artifact.into_parts();
                        if let Some(item) = item {
                            self.item = Some(item);
                        }
                        self.child = child_de;
                        return Ok(DeserializerOutput {
                            artifact: DeserializerArtifact::Deserializer(self),
                            event: DeserializerEvent::None,
                            allow_any: false,
                        });
                    } else {
                        // This is not the entity we expect. Skip it (and its
                        // subtree for a Start) by letting the orchestrator
                        // handle it.
                        return Ok(DeserializerOutput {
                            artifact: DeserializerArtifact::Deserializer(self),
                            event: DeserializerEvent::None,
                            allow_any: true,
                        });
                    }
                }
                Event::End(e) if e.name().as_ref() == b"IntuitResponse" => {
                    // End of IntuitResponse.
                    let item = self.item.ok_or_else(|| {
                        quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
                            "Expected exactly one item in IntuitResponse".to_string(),
                        ))
                    })?;
                    return Ok(DeserializerOutput {
                        artifact: DeserializerArtifact::Data(QBResponse(item)),
                        event: DeserializerEvent::None,
                        allow_any: false,
                    });
                }
                _ =>
                // Skip any non-start events inside <IntuitResponse> that we are not delegating.
                {
                    return Ok(DeserializerOutput {
                        artifact: DeserializerArtifact::Deserializer(self),
                        event: DeserializerEvent::None,
                        allow_any: false,
                    });
                }
            }
        }
    }

    fn finish(
        self,
        _helper: &mut DeserializeHelper,
    ) -> Result<QBResponse<T>, quickbooks_xsd::types::quick_xml::Error> {
        // We should have received the end event and returned in the `next` function.
        // If we get here, it means the XML ended without the closing IntuitResponse.
        Err(quickbooks_xsd::types::quick_xml::Error::custom(QBDeError(
            "Unexpected end of XML while inside IntuitResponse".to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::prelude::{Customer, Invoice};

    #[test]
    fn test_customer_deserializer() {
        let xml = r#"<Customer domain="QBO" sparse="false">
            <Id>239</Id>
            <SyncToken>0</SyncToken>
            <DisplayName>Test Customer</DisplayName>
        </Customer>"#;
        use quickbooks_xsd::types::quick_xml::{DeserializeSync, IoReader};
        let mut reader = IoReader::new(std::io::BufReader::new(xml.as_bytes()));
        let _cust: Customer = Customer::deserialize(&mut reader).unwrap();
    }

    #[test]
    fn test_qb_response_deserializer() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
            <Customer domain="QBO" sparse="false">
                <Id>239</Id>
                <SyncToken>0</SyncToken>
                <DisplayName>Test Customer</DisplayName>
            </Customer>
        </IntuitResponse>"#;
        use super::QBResponse;
        let response: QBResponse<Customer> = crate::xml_deserialize(xml.as_bytes()).unwrap();
        assert_eq!(response.0.id(), Some(&"239".to_string()));
        assert_eq!(
            response.0.display_name(),
            Some(&"Test Customer".to_string())
        );
    }

    #[test]
    fn test_qb_response_deserializer_invoice() {
        let xml = include_str!("../../test/data/query_response.xml");
        use super::QBResponse;
        let response: QBResponse<Invoice> = crate::xml_deserialize(xml.as_bytes()).unwrap();
        let inv = &response.0;
        assert_eq!(inv.id(), Some(&"239".to_string()));
        assert_eq!(inv.doc_number(), Some(&"1070".to_string()));
        assert_eq!(inv.total_amt(), Some(&150.00));
    }

    #[test]
    fn test_qb_response_deserializer_empty_intuit_response() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00"/>
                          </IntuitResponse>"#;
        use super::QBResponse;
        let result: Result<QBResponse<Invoice>, _> = crate::xml_deserialize(xml.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_qb_response_deserializer_wrong_entity() {
        let xml = r#"<IntuitResponse xmlns="http://schema.intuit.com/finance/v3" time="2026-06-05T08:11:46.819-07:00">
            <Vendor domain="QBO" sparse="false">
                <Id>456</Id>
                <SyncToken>0</SyncToken>
                <DisplayName>Test Vendor</DisplayName>
            </Vendor>
        </IntuitResponse>"#;
        use super::QBResponse;
        let result: Result<QBResponse<Customer>, _> = crate::xml_deserialize(xml.as_bytes());
        assert!(result.is_err());
    }
}
