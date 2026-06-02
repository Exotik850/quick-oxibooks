pub mod prelude {
    pub use quickbooks_xsd::finance::LineType as Line;
    pub use quickbooks_xsd::intuit_rest_service_def::{
        Account, Attachable, Bill, BillPayment,
        BooleanTypeCustomFieldDefinition, Budget, ChargeCredit, Class, Company, CompanyCurrency,
        CompanyInfo, CreditCardPaymentTxn, CreditMemo, CustomFieldDefinition, Customer,
        CustomerType, DateTypeCustomFieldDefinition, Department, Deposit, EmailDeliveryInfo,
        ErrorType, Estimate, ExchangeRate, FixedAsset, InventoryAdjustment, InventorySite, Invoice,
        Item, JournalCode, JournalEntry, MasterAccount, NumberTypeCustomFieldDefinition, OtherName,
        Payment, PaymentMethod, Preferences, PriceLevel, PurchaseOrder, RecurringTransaction,
        RefundReceipt, ReimburseCharge, SalesOrder, SalesReceipt, SalesRep, StatementCharge,
        Status, StringTypeCustomFieldDefinition, SyncActivity, Tag, Task, TaxAgency,
        TaxClassification, TaxCode, TaxPayment, TaxRate, TaxReturn, TdsMetadata, Term, Transfer,
        UserAlert, Vendor, VendorCredit, VendorType,
    };

    pub use quickbooks_xsd::intuit_base_types::ModificationMetaDataType as Metadata;
}

pub use quickbooks_xsd::*;
