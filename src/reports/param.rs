use chrono::NaiveDate;
use quickbooks_types::SummarizeColumnsByEnum;

use crate::types::DateMacro;

struct CustomField<const N: usize>(String);

impl<const N: usize> CustomField<N> {
  const check: () = assert!(N >= 1 && N <= 3, "CustomField N must be 1, 2, or 3");
}

impl<const N: usize> QBReportParam for CustomField<N> {
    fn name() -> &'static str {
        match N {
            1 => "custom1",
            2 => "custom2",
            3 => "custom3",
            _ => "custom",
        }
    }
    fn value(&self) -> String {
        self.0.to_string()
    }
}



pub trait QBReportParam {
    fn name() -> &'static str;
    fn value(&self) -> String;
}

impl QBReportParam for DateMacro {
  fn name() -> &'static str {
      "date_macro"
  }
  fn value(&self) -> String {
      self.to_string()
  }
}

impl QBReportParam for SummarizeColumnsByEnum {
  fn name() -> &'static str {
      "summarize_columns_by"
  }
  fn value(&self) -> String {
      self.to_string()
  }
}

macro_rules! impl_report_param  {
    ($($str_name:ident - $query:expr ; $val:ident);+ $(;)?) => {
        $(
            pub struct $str_name(pub $val);
            impl QBReportParam for $str_name {
                fn name() -> &'static str {
                    $query
                }
                fn value(&self) -> String {
                    self.0.to_string()
                }
            }
        )+
    };
}

impl_report_param!(
  AccountingMethod - "accounting_method"; String;
  StartDate - "start_date"; NaiveDate;
  EndDate - "end_date"; NaiveDate;
  AsOfDate - "as_of_date"; NaiveDate;
  Customer - "customer"; u32;
  Vendor - "vendor"; u32;

);