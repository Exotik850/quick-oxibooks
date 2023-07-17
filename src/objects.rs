use chrono::{NaiveDate, DateTime, Utc};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;

use crate::quickbook::{Line, LinkedTxn, Email, Addr, MetaData, NtRef};

#[derive(Eq,PartialEq,Debug)]
pub(crate) struct RefreshToken(String);

impl RefreshToken {
    pub fn get() -> Self {
        let token = fs::read_to_string("refresh.txt");
        match token {
            Ok(tok) => {Self(tok)}
            Err(e) => {panic!("Could not retreive refresh token from file:\n\t{e:?}")}
        }
    }
}

impl Drop for RefreshToken {
    fn drop(&mut self) {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open("refresh.txt")
            .expect("Cannot open refresh token file");
        file.write_all(self.0.as_bytes())
            .expect("Could not write refresh token bytes");
        println!("Dropped refresh token successfully");
    }
}




#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct Invoice {
    #[serde(default)]
    txn_date: NaiveDate,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    print_status: String,
    #[serde(default)]
    sales_term_ref: NtRef,
    #[serde(default)]
    total_amt: f32,
    #[serde(default)]
    line: Vec<Line>,
    #[serde(default)]
    due_date: NaiveDate,
    #[serde(default)]
    sparse: bool,
    #[serde(default)]
    doc_number: String,
    #[serde(default)]
    customer_ref: NtRef,
    #[serde(default)]
    txn_tax_detail: TxnTaxDetail,
    #[serde(default)]
    sync_token: String,
    #[serde(default)]
    linked_txn: Vec<LinkedTxn>,
    #[serde(default)]
    bill_email: Email,
    #[serde(default)]
    ship_addr: Addr,
    #[serde(default)]
    email_status: String,
    #[serde(default)]
    bill_addr: Addr,
    #[serde(default)]
    meta_data: MetaData,
    #[serde(default)]
    custom_field: Vec<CustomField>,
    #[serde(default)]
    id: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CustomField {

}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, Default)]
#[serde(rename_all="PascalCase")]
struct TxnTaxDetail {
    #[serde(default)]
    txn_tax_code_ref: NtRef,
    #[serde(default)]
    total_tax: f32,
    #[serde(default)]
    tax_line: Line,
}


#[derive(Deserialize)]
pub(crate) struct InvoiceResponse {
    pub(crate) invoice: Invoice,
    time: DateTime<Utc>
}