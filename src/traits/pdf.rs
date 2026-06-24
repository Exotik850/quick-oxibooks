use quickbooks_xsd::intuit_rest_service_def::*;

use crate::traits::QBItem;

pub trait QBPDFAvailable: QBItem {}

macro_rules! impl_pdf {
    ($($ty:ty),* $(,)?) => {
        $(
            impl QBPDFAvailable for $ty {}
        )*
    };
}

impl_pdf!(
    Invoice,
    RefundReceipt,
    SalesReceipt,
    Payment,
    CreditMemo,
    Estimate
);
