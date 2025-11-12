//! Module for `QuickBooks` report retrieval functions.

use quickbooks_types::reports::{types::QBReportType, Report};
use ureq::{http::Method, Agent};

use crate::{functions::qb_request, APIResult, QBContext};

/// Trait for retrieving `QuickBooks` financial reports.
///
/// This trait provides the interface for fetching various financial reports from
/// `QuickBooks`, such as Profit & Loss, Balance Sheet, Trial Balance, and others.
/// Reports can be customized with parameters like date ranges, summarization levels,
/// and accounting methods.
///
/// # Automatic Implementation
///
/// This trait is implemented for the [`Report`] struct from `quickbooks_types::reports`.
/// You don't need to implement it manually.
///
/// # Report Types
///
/// `QuickBooks` supports many report types including:
/// - **`ProfitAndLoss`**: Income statement showing revenue and expenses
/// - **`BalanceSheet`**: Statement of financial position
/// - **`TrialBalance`**: List of all account balances
/// - **`CashFlow`**: Cash flow statement
/// - **`AgedReceivables`**: Outstanding customer invoices by age
/// - **`AgedPayables`**: Outstanding vendor bills by age
/// - **`GeneralLedger`**: Detailed transaction history
/// - And many more...
///
/// # Examples
///
/// ## Basic Report (No Parameters)
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::reports::QBReport;
/// use quickbooks_types::reports::{Report, types::ProfitAndLoss};
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
/// // Get a basic Profit & Loss report
/// let report = Report::get(
///     &qb_context,
///     &client,
///     &ProfitAndLoss,
///     None
/// ).unwrap();
/// println!("Report name: {:?}", report.header.as_ref().unwrap().report_name);
/// ```
///
/// ## Report with Parameters
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::reports::QBReport;
/// use quickbooks_types::reports::{Report, types::{BalanceSheet, BalanceSheetParams}};
/// use quickbooks_types::reports::params::SummarizeColumnBy;
/// use chrono::NaiveDate;
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
/// // Create parameters for the report
/// let params = BalanceSheetParams::default()
///         .start_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
///         .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
///         .summarize_column_by(SummarizeColumnBy::Month);
///
/// // Get Balance Sheet with parameters
/// let report = Report::get(
///     &qb_context,
///     &client,
///     &BalanceSheet,
///     Some(params)
/// ).unwrap();
/// println!("Report name: {:?}", report.header.as_ref().unwrap().report_name);
/// ```
///
/// ## Available Report Parameters
///
/// Common parameters across reports include:
/// - **Date Ranges**: `start_date`, `end_date` for filtering by date
/// - **Summarization**: `summarize_column_by` (Day, Week, Month, Quarter, Year)
/// - **Accounting Method**: `accounting_method` (Cash, Accrual)
/// - **Customers/Classes**: Filter by specific customers or classes
/// - **Departments**: Filter by department if using department tracking
///
/// # Return Value
///
/// Returns a [`Report`] struct containing:
/// - Report metadata (name, date range, accounting method, etc.)
/// - Hierarchical data structure with rows and columns
/// - Financial data organized by accounts and time periods
/// - Summary totals and subtotals
///
/// # Errors
///
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: Invalid report parameters or `QuickBooks` API error
/// - `JsonError`: Response parsing errors
/// - Rate limiting errors if API limits are exceeded
///
/// # Performance Notes
///
/// - Large date ranges may result in slower response times
/// - Complex reports with many parameters may take longer to generate
/// - Consider using summary levels (Month, Quarter) for large datasets
/// - Some reports may timeout if the data set is too large
pub trait QBReport {
    fn get<T: QBReportType>(
        qb: &QBContext,
        client: &Agent,
        report_type: &T,
        params: Option<T::QueryParams>,
    ) -> APIResult<Self>
    where
        Self: Sized;
}

impl QBReport for Report {
    fn get<T: QBReportType>(
        qb: &QBContext,
        client: &Agent,
        report_type: &T,
        params: Option<T::QueryParams>,
    ) -> APIResult<Self> {
        let path = format!(
            "/v3/company/{}/reports/{}",
            qb.company_id,
            report_type.url_name()
        );
        qb_request(
            qb,
            client,
            Method::GET,
            dbg!(&path),
            None::<&()>,
            Some("application/json"),
            params
                .as_ref()
                .map(quickbooks_types::reports::types::QBReportParams::params),
        )
    }
}
