//! Functions for querying `QuickBooks` entities using SQL-like syntax.

use std::fmt::{Display, Write};

use quickbooks_xsd::types::quick_xml::WithDeserializer;
// use quickbooks_types::QBItem;
use serde::Deserialize;
use ureq::{http::Method, Agent};

use crate::{error::APIError, traits::QBItem, APIResult, QBContext};

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
impl quickbooks_xsd::types::quick_xml::WithSerializer for Empty {
    type Serializer<'x> = Empty;
    fn serializer<'ser>(
        &'ser self,
        _: Option<&'ser str>,
        _: bool,
    ) -> Result<Self::Serializer<'ser>, quickbooks_xsd::types::quick_xml::Error> {
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

/// Unsafe function to query the quickbooks context using the raw query string
///
/// # Safety
///
/// This function is unsafe because it allows arbitrary query strings to be executed
/// against the QuickBooks API. Malformed or malicious query strings could lead to
/// unexpected behavior or security vulnerabilities. It is the caller's responsibility
/// to ensure that the query string is well-formed and safe to execute.
pub unsafe fn qb_query_raw<T: QBItem>(
    query_str: impl Display,
    qb: &QBContext,
    client: &Agent,
) -> Result<Vec<T>, APIError> {
    let response: QueryResponseExt<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/query", qb.company_id),
        None::<&Empty>,
        None,
        Some([("query", format!("{query_str}"))]),
    )?;
    #[cfg(feature = "logging")]
    log::info!("Successfully Queried Raw Data for query string : {query_str}",);
    Ok(response.query_response.items)
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
    let mut query = format!("select * from {} {query_str}", T::NAME);
    if let Some(max) = max_results {
        write!(&mut query, " MAXRESULTS {max}").expect("Writing to string should not fail");
    }
    unsafe { qb_query_raw::<T>(query, qb, client) }
}

/*
#[derive(Clone, Debug)]
    pub struct QueryResponseType {
        pub start_position: ::core::option::Option<::core::primitive::i32>,
        pub max_results: ::core::option::Option<::core::primitive::i32>,
        pub total_count: ::core::option::Option<::core::primitive::i32>,
        pub content: ::std::vec::Vec<QueryResponseTypeContent>,
    }
    #[derive(Clone, Debug)]
    pub enum QueryResponseTypeContent {
        Warnings(WarningsType),
        IntuitObject(IntuitObject),
        Fault(FaultType),
    }
    #[derive(Clone, Debug)]
    pub enum IntuitObject {
        RecurringTransaction(RecurringTransactionType),
        Customer(super::intuit_names_types::CustomerType),
        CustomerType(super::intuit_names_types::CustomerTypeType),
        Employee(super::intuit_names_types::EmployeeType),
        Vendor(super::intuit_names_types::VendorType),
        OtherName(super::intuit_names_types::OtherNameType),
        Estimate(super::finance::EstimateType),
        SalesOrder(super::finance::SalesOrderType),
        SalesReceipt(super::finance::SalesReceiptType),
        PurchaseOrder(super::finance::PurchaseOrderType),
        Purchase(super::finance::PurchaseType),
        BillPayment(super::finance::BillPaymentType),
        Payment(super::finance::PaymentType),
        CreditMemo(super::finance::CreditMemoType),
        StatementCharge(super::finance::StatementChargeType),
        ChargeCredit(super::finance::ChargeCreditType),
        CreditCardPaymentTxn(super::finance::CreditCardPaymentTxnType),
        ReimburseCharge(super::finance::ReimburseChargeType),
        PaymentMethod(super::finance::PaymentMethodType),
        Term(super::finance::TermType),
        JournalEntry(super::finance::JournalEntryType),
        Transfer(super::finance::TransferType),
        Deposit(super::finance::DepositType),
        Class(super::finance::ClassType),
        TimeActivity(super::finance::TimeActivityType),
        Item(super::finance::ItemType),
        TaxClassification(super::finance::TaxClassificationType),
        TaxCode(super::finance::TaxCodeType),
        TaxPayment(super::finance::TaxPaymentType),
        TaxReturn(super::finance::TaxReturnType),
        TaxRate(super::finance::TaxRateType),
        VendorCredit(super::finance::VendorCreditType),
        SalesRep(super::finance::SalesRepType),
        InventorySite(super::finance::InventorySiteType),
        Task(super::finance::TaskType),
        UserAlert(super::finance::UserAlertType),
        VendorType(super::intuit_names_types::VendorTypeType),
        PriceLevel(super::finance::PriceLevelType),
        Company(super::finance::CompanyType),
        CompanyInfo(super::finance::CompanyInfoType),
        FixedAsset(super::finance::FixedAssetType),
        RefundReceipt(super::finance::RefundReceiptType),
        Account(super::finance::AccountType),
        Preferences(super::finance::PreferencesType),
        Invoice(super::finance::InvoiceType),
        EmailDeliveryInfo(super::finance::EmailDeliveryInfoType),
        Department(super::finance::DepartmentType),
        Bill(super::finance::BillType),
        Attachable(super::finance::AttachableType),
        InventoryAdjustment(super::finance::InventoryAdjustmentType),
        BooleanTypeCustomFieldDefinition(
            super::intuit_base_types::BooleanTypeCustomFieldDefinitionType,
        ),
        CustomFieldDefinition(super::intuit_base_types::CustomFieldDefinitionType),
        DateTypeCustomFieldDefinition(super::intuit_base_types::DateTypeCustomFieldDefinitionType),
        NumberTypeCustomFieldDefinition(
            super::intuit_base_types::NumberTypeCustomFieldDefinitionType,
        ),
        StringTypeCustomFieldDefinition(
            super::intuit_base_types::StringTypeCustomFieldDefinitionType,
        ),
        Status(StatusType),
        SyncActivity(SyncActivityType),
        Budget(super::finance::BudgetType),
        TaxAgency(super::intuit_names_types::TaxAgencyType),
        TdsMetadata(super::finance::TdsMetadataType),
        CompanyCurrency(super::finance::CompanyCurrencyType),
        ExchangeRate(super::finance::ExchangeRateType),
        JournalCode(super::finance::JournalCodeType),
        QbdtEntityIdMapping(super::finance::QbdtEntityIdMappingType),
        MasterAccount(super::finance::MasterAccountType),
        Tag(super::finance::TagType),
        TaxService(super::sales_tax::TaxServiceType),
    }
*/

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone, Default)]
// #[serde(rename_all = "PascalCase", default)]
struct QueryResponse<T> {
    total_count: i64,
    // #[serde(
    //     alias = "Item",
    //     alias = "Account",
    //     alias = "Invoice",
    //     alias = "Attachable",
    //     alias = "Bill",
    //     alias = "CompanyInfo",
    //     alias = "Customer",
    //     alias = "Employee",
    //     alias = "Estimate",
    //     alias = "Payment",
    //     alias = "SalesReceipt",
    //     alias = "Vendor"
    // )]
    items: Vec<T>,
    start_position: i64,
    max_results: i64,
}

#[derive(Debug)]
pub struct QueryResponseDeserializer<T: WithDeserializer + std::fmt::Debug> {
  inner: <T as WithDeserializer>::Deserializer,
  start_position: i64,
  max_results: i64,
  total_count: i64,
}

impl<'de, D: WithDeserializer + std::fmt::Debug> quickbooks_xsd::types::quick_xml::Deserializer<'de, QueryResponse<D>> for QueryResponseDeserializer<D> {
    fn init(helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper, event: quick_xml::events::Event<'de>) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QueryResponse<D>> {
        todo!()
    }

    fn next(self, helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper, event: quick_xml::events::Event<'de>) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QueryResponse<D>> {
        todo!()
    }

    fn finish(self, helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper) -> Result<QueryResponse<D>, quickbooks_xsd::types::quick_xml::Error> {
        todo!()
    }
}

impl<D: WithDeserializer + std::fmt::Debug> WithDeserializer for QueryResponse<D> {
    type Deserializer = QueryResponseDeserializer<D>;
}

#[derive(Debug)]
struct QueryResponseExtDeserializer<T: WithDeserializer + std::fmt::Debug> {
    inner: QueryResponseDeserializer<T>,
    time: String,
}

impl<'de, T: WithDeserializer + std::fmt::Debug> quickbooks_xsd::types::quick_xml::Deserializer<'de, QueryResponseExt<T>> for QueryResponseExtDeserializer<T> {
    fn init(helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper, event: quick_xml::events::Event<'de>) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QueryResponseExt<T>> {
        todo!()
    }

    fn next(self, helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper, event: quick_xml::events::Event<'de>) -> quickbooks_xsd::types::quick_xml::DeserializerResult<'de, QueryResponseExt<T>> {
        todo!()
    }

    fn finish(self, helper: &mut quickbooks_xsd::types::quick_xml::DeserializeHelper) -> Result<QueryResponseExt<T>, quickbooks_xsd::types::quick_xml::Error> {
        todo!()
    }
}

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone)]
struct QueryResponseExt<T> {
    // #[serde(default, rename = "QueryResponse")]
    query_response: QueryResponse<T>,
    #[allow(dead_code)]
    time: String,
}

impl<T: WithDeserializer + std::fmt::Debug> WithDeserializer for QueryResponseExt<T> {
    type Deserializer = QueryResponseExtDeserializer<T>;
}
