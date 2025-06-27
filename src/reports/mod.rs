mod param;

pub trait QBReport {}



pub trait HasQBReportType {
    fn url_name() -> &'static str;
    fn valid_query_params() -> &'static [&'static str];
}

macro_rules! impl_report_type {
    ($($report_ty:ident, $url_name:expr, [$($param:tt),* $(,)?];)* $(;)?) => {
      $(
          pub struct $report_ty;
        impl HasQBReportType for $report_ty {
            fn url_name() -> &'static str {
                $url_name
            }

            fn valid_query_params() -> &'static [&'static str] {
                &[$($param),*]
            }
        }
      )+
    };
    () => {}
}

impl_report_type!(
  AccountListDetail, "AccountList", ["accounting_method", "date_macro", "start_date", "end_date", "summarize_column_by"];
  APAgingDetail, "AgedPayableDetail", ["as_of_date", "aging_method", "vendor", "columns"];
  APAgingSummary, "AgedPayables", ["customer", "qzurl", "vendor", "date_macro", "department", "report_date", "sort_order", "aging_method"];
  ARAgingDetail, "AgedReceivableDetail", ["customer", "shipvia", "term", "end_duedate", "start_duedate", "custom1", "custom2", "custom3", "report_date", "num_periods", "aging_method", "past_due", "aging_period", "columns"];
  ARAgingSummary, "AgedReceivables", ["customer", "qzurl", "date_macro", "aging_method", "report_date", "sort_order", "department"];
  BalanceSheet, "BalanceSheet", ["customer", "qzurl", "end_date", "accounting_method", "date_macro", "adjusted_gain_loss", "class", "item", "sort_order", "summarize_column_by", "department", "vendor", "start_date"];
  CashFlow, "CashFlow", ["customer", "vendor", "end_date", "date_macro", "class", "item", "sort_order", "summarize_column_by", "department", "start_date"];
  CustomerBalance, "CustomerBalance", ["customer", "accounting_method", "date_macro", "arpaid", "report_date", "sort_order", "summarize_column_by", "department"];
  CustomerBalanceDetail, "CustomerBalanceDetail", ["customer", "shipvia", "term", "end_duedate", "start_duedate", "custom1", "custom2", "custom3", "arpaid", "report_date", "sort_order", "aging_method", "department"];
  CustomerIncome, "CustomerIncome", ["customer", "term", "accounting_method", "end_date", "date_macro", "class", "sort_order", "summarize_column_by", "department", "start_date", "vendor"];
  FECReport, "FECReport", ["attachmentType", "withQboIdentifier", "start_date", "end_date", "add_due_date"];
  GeneralLedger, "GeneralLedger", ["customer", "account", "accounting_method", "source_account", "end_date", "date_macro", "account_type", "sort_by", "sort_order", "start_date", "summarize_column_by", "class", "item", "department", "vendor", "columns"];
  GeneralLedgerFR, "GeneralLedgerFR", ["customer", "account", "accounting_method", "source_account", "end_date", "date_macro", "account_type", "sort_by", "sort_order", "start_date", "summarize_column_by", "class", "vendor"];
  InventoryValuationDetail, "InventoryValuationDetail", ["end_date", "end_svcdate", "date_macro", "svcdate_macro", "start_svcdate", "group_by", "start_date", "columns"];
  InventoryValuationSummary, "InventoryValuationSummary", ["qzurl", "date_macro", "item", "report_date", "sort_order", "summarize_column_by"];
  JournalReport, "JournalReport", ["end_date", "date_macro", "sort_by", "sort_order", "start_date", "columns"];
  ProfitAndLoss, "ProfitAndLoss", ["customer", "qzurl", "accounting_method", "end_date", "date_macro", "adjusted_gain_loss", "class", "item", "sort_order", "summarize_column_by", "department", "vendor", "start_date"];
  ProfitAndLossDetail, "ProfitAndLossDetail", ["customer", "account", "accounting_method", "end_date", "date_macro", "adjusted_gain_loss", "class", "sort_by", "payment_method", "sort_order", "employee", "department", "vendor", "account_type", "start_date", "columns"];
  SalesByClassSummary, "ClassSales", ["customer", "accounting_method", "end_date", "date_macro", "class", "item", "summarize_column_by", "department", "start_date"];
  SalesByCustomer, "CustomerSales", ["customer", "qzurl", "accounting_method", "end_date", "date_macro", "class", "item", "sort_order", "summarize_column_by", "department", "start_date"];
  SalesByDepartment, "DepartmentSales", ["customer", "accounting_method", "end_date", "date_macro", "class", "item", "sort_order", "summarize_column_by", "department", "start_date"];
  SalesByProduct, "ItemSales", ["customer", "end_duedate", "accounting_method", "end_date", "date_macro", "start_duedate", "class", "item", "sort_order", "summarize_column_by", "department", "start_date"];
  TaxSummary, "TaxSummary", ["agency_id", "accounting_method", "end_date", "date_macro", "sort_order", "start_date"];
  TransactionList, "TransactionList", ["date_macro", "payment_method", "duedate_macro", "arpaid", "bothamount", "transaction_type", "docnum", "start_moddate", "source_account_type", "group_by", "start_date", "department", "start_duedate", "columns", "end_duedate", "vendor", "end_date", "memo", "appaid", "moddate_macro", "printed", "createdate_macro", "cleared", "customer", "qzurl", "term", "end_createdate", "name", "sort_by", "sort_order", "start_createdate", "end_moddate"];
  TransactionListByCustomer, "TransactionListByCustomer", ["date_macro", "payment_method", "duedate_macro", "arpaid", "bothamount", "transaction_type", "docnum", "start_moddate", "source_account_type", "group_by", "start_date", "department", "start_duedate", "columns", "end_duedate", "end_date", "memo", "appaid", "moddate_macro", "printed", "createdate_macro", "cleared", "customer", "qzurl", "term", "end_createdate", "name", "sort_by", "sort_order", "start_createdate", "end_moddate"];
  TransactionListByVendor, "TransactionListByVendor", ["date_macro", "payment_method", "duedate_macro", "arpaid", "bothamount", "transaction_type", "docnum", "start_moddate", "source_account_type", "group_by", "start_date", "department", "start_duedate", "columns", "end_duedate", "vendor", "end_date", "memo", "appaid", "moddate_macro", "printed", "createdate_macro", "cleared", "qzurl", "term", "end_createdate", "name", "sort_by", "sort_order", "start_createdate", "end_moddate"];
  TransactionListWithSplits, "TransactionListWithSplits", ["docnum", "name", "end_date", "date_macro", "payment_method", "source_account_type", "transaction_type", "group_by", "sort_by", "sort_order", "start_date", "columns"];
  TrialBalance, "TrialBalance", ["accounting_method", "end_date", "date_macro", "sort_order", "summarize_column_by", "start_date"];
  VendorBalance, "VendorBalance", ["qzurl", "accounting_method", "date_macro", "appaid", "report_date", "sort_order", "summarize_column_by", "department", "vendor"];
  VendorBalanceDetail, "VendorBalanceDetail", ["term", "accounting_method", "date_macro", "appaid", "report_date", "sort_order", "summarize_column_by", "department", "vendor", "columns", "report_date", "duedate_macro", "start_duedate", "end_duedate"];
  VendorExpenses, "VendorExpenses", ["customer", "vendor", "end_date", "date_macro", "class", "sort_order", "summarize_column_by", "department", "accounting_method", "start_date"];
);
