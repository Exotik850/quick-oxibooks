pub use crate::types::prelude::*;
use quickbooks_xsd::types::quick_xml::{WithDeserializer, WithSerializer};

mod create;
pub use create::QBCreatable;
mod to_ref;
pub use to_ref::{NoIdForRef, QBToAttachableRef, QBToRef};

#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "pdf")]
pub use pdf::QBPDFAvailable;

pub trait QBItem:
    WithSerializer + WithDeserializer + Clone + Sized + Send + std::fmt::Debug
{
    const NAME: &'static str;
    const QB_ID: &'static str;

    fn id(&self) -> Option<&str>;
    fn sync_token(&self) -> Option<&str>;
    fn meta_data(&self) -> Option<&Metadata>;
}

macro_rules! impl_qb_data {
    ($qb_ty:ty, $lt:lifetime, $id:expr, $sync:expr, $meta:expr) => {
        impl QBItem for $qb_ty {
            const NAME: &'static str = stringify!($qb_ty);
            const QB_ID: &'static str = paste::paste! { stringify!([< $qb_ty:lower >]) };

            fn id<$lt>(& $lt self) -> Option<& $lt str> {
                ($id)(self)
            }

            fn sync_token<$lt>(& $lt self) -> Option<& $lt str> {
                ($sync)(self)
            }

            fn meta_data<$lt>(& $lt self) -> Option<& $lt Metadata> {
                ($meta)(self)
            }
        }
    };
}

macro_rules! impl_qb_flat {
    ($flat_ty:ty) => {
        impl_qb_data!(
            $flat_ty,
            'a,
            |x: &'a $flat_ty| x.id().map(|s| s.as_str()),
            |x: &'a $flat_ty| x.sync_token().map(|s| s.as_str()),
            |x: &'a $flat_ty| x.meta_data()
        );
    };
    ($($flat_ty:ty),*) => {
        $(
            impl_qb_flat!($flat_ty);
        )*
    };
}

macro_rules! impl_qb_simple {
    ($simple:ty) => {
        impl_qb_data!(
            $simple,
            'a,
            |x: &'a $simple| x.id.as_deref(),
            |x: &'a $simple| x.sync_token.as_deref(),
            |x: &'a $simple| x.meta_data.as_ref()
        );
    };
    ($($simple:ty),*) => {
        $(
            impl_qb_simple!($simple);
        )*
    };
}

impl_qb_flat!(
    CreditMemo,
    Customer,
    Estimate,
    Invoice,
    Payment,
    PriceLevel,
    PurchaseOrder,
    RefundReceipt,
    SalesOrder,
    SalesReceipt,
    SalesRep,
    Term,
    BillPayment
);
impl_qb_simple!(
    Budget,
    ChargeCredit,
    Class,
    CreditCardPaymentTxn,
    CustomerType,
    Department,
    Deposit,
    EmailDeliveryInfo,
    ExchangeRate,
    FixedAsset,
    JournalCode,
    JournalEntry,
    MasterAccount,
    OtherName,
    PaymentMethod,
    RecurringTransaction,
    ReimburseCharge,
    StatementCharge,
    Status,
    SyncActivity,
    Tag,
    Task,
    TdsMetadata,
    Transfer,
    UserAlert,
    VendorCredit,
    Account,
    Attachable,
    Bill,
    BooleanTypeCustomFieldDefinition,
    Company,
    CompanyCurrency,
    CompanyInfo,
    CustomFieldDefinition,
    DateTypeCustomFieldDefinition,
    InventoryAdjustment,
    InventorySite,
    Item,
    NumberTypeCustomFieldDefinition,
    Preferences,
    StringTypeCustomFieldDefinition,
    TaxAgency,
    TaxClassification,
    TaxCode,
    TaxRate,
    TaxReturn,
    Vendor
);
