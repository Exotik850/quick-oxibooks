/*!
 * A rust library for interacting with the QuickBooks API.
 *
 * For more information, you can check out their documentation at:
 * https://developer.intuit.com/app/developer/qbo/docs/develop
 *
 * Example:
 *
 * ORIGINIALLY FROM https://github.com/oxidecomputer/cio 
 * LICENSED UNDER APACHE 2.0
 * 
 * ```ignore
 * use quickbooks::QuickBooks;
 * use serde::{Deserialize, Serialize};
 *
 * async fn list_purchases() {
 *     // Initialize the QuickBooks client.
 *     let quickbooks = QuickBooks::new_from_env("", "", "");
 *
 *     let purchases = quickbooks.list_purchases().await.unwrap();
 *
 *     println!("{:?}", purchases);
 * }
 * ```
 */
use std::fmt::Display;
#[allow(dead_code)]

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use intuit_oauth::{AuthClient, AuthorizeType, Unauthorized, Authorized};
use paste::paste;
use quickbooks_types::models::{MetaData, LinkedTxn, Invoice, Item, NtRef, Bill, WebAddr, CompanyInfo};
use reqwest::{header, Client, Method, Request, StatusCode, Url};
use serde::{Deserialize, Serialize};

/// Endpoint for the QuickBooks API.
const ENDPOINT: &str = "https://sandbox-quickbooks.api.intuit.com/v3/";

const QUERY_PAGE_SIZE: i64 = 1000;

pub struct APIError {
    pub status_code: StatusCode,
    pub body: String,
}

/// Entrypoint for interacting with the QuickBooks API.
#[derive(Debug, Clone)]
pub struct QuickBooks<T>
where T: AuthorizeType
{
    redirect_uri: String,
    company_id: String,
    client: Arc<AuthClient<T>>,
    http_client: Arc<Client>,
}

impl QuickBooks<Unauthorized> {
    /// Create a new QuickBooks client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key your requests will work.
    pub async fn new<I, K, B, R>(
        client_id: I,
        client_secret: K,
        company_id: B,
        redirect_uri: R,
    ) -> QuickBooks<Authorized>
    where
    I: Display,
    K: Display,
        B: Display,
        R: Display,
    {
        let client = AuthClient::new(&client_id, &client_secret, &redirect_uri,
            &company_id, intuit_oauth::Environment::SANDBOX).await;
        
        let mut client = client.authorize().await;

        let qb = QuickBooks {
            company_id: company_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            client: Arc::new(client),
            http_client: Arc::new(Client::new())
        };

        qb
    }        
    


    /// Create a new QuickBooks client struct from environment variables. It
    /// takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key and your requests will work.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env<C>(company_id: C) -> QuickBooks<Authorized>
    where
        C: Display,
    {
        let redirect_uri = dotenv::var("INTUIT_REDIRECT_URI").unwrap();
        let client = AuthClient::new_from_env(&company_id, intuit_oauth::Environment::SANDBOX).await;
        let mut client = client.authorize().await;
        client.refresh_access_token().await;


        QuickBooks { 
            redirect_uri,
            company_id: company_id.to_string(),
            client: Arc::new(client),
            http_client: Arc::new(Client::new())
        }
    }
}

impl QuickBooks<Authorized>
{

    fn request<B>(
        &self,
        method: Method,
        path: &str,
        body: B,
        query: Option<&[(&str, &str)]>,
    ) -> Request
    where
        B: Serialize,
    {
        let base = Url::parse(ENDPOINT).unwrap();
        let url = base.join(path).unwrap();

        let bt = format!("Bearer {}", self.client.data.access_token.secret());
        let bearer = header::HeaderValue::from_str(&bt).unwrap();

        // Set the default headers.
        let mut headers = header::HeaderMap::new();
        headers.append(header::AUTHORIZATION, bearer);
        headers.append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let mut rb = self.http_client.request(method.clone(), url).headers(headers);

        if let Some(val) = query {
            rb = rb.query(&val);
        }

        // Add the body, this is to ensure our GET and DELETE calls succeed.
        if method != Method::GET && method != Method::DELETE {
            rb = rb.json(&body);
        }

        // Build the request.
        rb.build().unwrap()
    }


    pub async fn company_info(&self, company_id: &str) -> Result<CompanyInfo, APIError> {
        // Build the request.
        let request = self.request(
            Method::GET,
            &format!("company/{company_id}/query"),
            (),
            Some(&[("query", "select * from CompanyInfo")]),
        );

        let resp = self.http_client.execute(request).await.unwrap();
        match resp.status() {
            StatusCode::OK => (),
            s => {
                return Err(APIError {
                    status_code: s,
                    body: resp.text().await.unwrap(),
                })
            }
        };

        let r: CompanyInfoResponse = resp.json().await.unwrap();

        Ok(r.query_response.company_info.get(0).unwrap().clone())
    }


    pub async fn get_invoice_by_doc_num(&self, doc_num: &str) -> Result<Invoice, APIError> {
        let request = self.request(
            Method::GET,
            &format!("company/{}/query", self.company_id),
            (),
            Some(&[(
                "query",
                &format!(
                    "select * from Invoice where DocNumber = '{doc_num}' MAXRESULTS {QUERY_PAGE_SIZE}"
                ),
            )]),
        );

        let resp = self.http_client.execute(request).await.unwrap();
        match resp.status() {
            StatusCode::OK => (),
            s => {
                return Err(APIError {
                    status_code: s,
                    body: resp.text().await.unwrap(),
                })
            }
        };

        let r: ItemsResponse = resp.json().await.unwrap();

        Ok(r.query_response.invoice[0].clone())
    }

    // pub async fn get_bill(&self, bill_id: &str) -> Result<Bill, APIError> {
    //     // Build the request.
    //     let request = self.request(
    //         Method::GET,
    //         &format!("company/{}/bill/{bill_id}", self.company_id),
    //         (),
    //         None,
    //     );

    //     let resp = self.client.execute(request).await.unwrap();
    //     match resp.status() {
    //         StatusCode::OK => (),
    //         s => {
    //             return Err(APIError {
    //                 status_code: s,
    //                 body: resp.text().await.unwrap(),
    //             })
    //         }
    //     };

    //     let r: BillResponse = resp.json().await.unwrap();

    //     Ok(r.bill)
    // }





    pub async fn list_items(&self) -> Result<Vec<Item>, APIError> {
        // Build the request.
        let request = self.request(
            Method::GET,
            &format!("company/{}/query", self.company_id),
            (),
            Some(&[(
                "query",
                &format!("SELECT * FROM Item MAXRESULTS {QUERY_PAGE_SIZE}"),
            )]),
        );

        let resp = self.http_client.execute(request).await.unwrap();
        match resp.status() {
            StatusCode::OK => (),
            s => {
                return Err(APIError {
                    status_code: s,
                    body: resp.text().await.unwrap(),
                })
            }
        };

        let items: ItemsResponse = resp.json().await.unwrap();

        Ok(items.query_response.item)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CountResponse {
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyInfoResponse {
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryResponse {
    #[serde(default, rename = "totalCount")]
    pub total_count: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Item")]
    pub item: Vec<Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Purchase")]
    pub purchase: Vec<Purchase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Attachable")]
    pub attachable: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "BillPayment")]
    pub bill_payment: Vec<BillPayment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "CompanyInfo")]
    pub company_info: Vec<CompanyInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Invoice")]
    pub invoice: Vec<Invoice>,
    #[serde(default, rename = "startPosition")]
    pub start_position: i64,
    #[serde(default, rename = "maxResults")]
    pub max_results: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsResponse {
    #[serde(default, rename = "QueryResponse")]
    pub query_response: QueryResponse,
    pub time: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    #[serde(default, rename = "AccountRef")]
    pub account_ref: NtRef,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "PaymentType"
    )]
    pub payment_type: String,
    #[serde(default, rename = "EntityRef")]
    pub entity_ref: NtRef,
    #[serde(default, rename = "TotalAmt")]
    pub total_amt: f32,
    #[serde(default, rename = "PurchaseEx")]
    pub purchase_ex: PurchaseEx,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub domain: String,
    pub sparse: bool,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "SyncToken"
    )]
    pub sync_token: String,
    #[serde(rename = "MetaData")]
    pub meta_data: MetaData,
    #[serde(rename = "TxnDate")]
    pub txn_date: NaiveDate,
    #[serde(default, rename = "CurrencyRef")]
    pub currency_ref: NtRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Line")]
    pub line: Vec<Line>,
    #[serde(default, rename = "Credit")]
    pub credit: bool,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "DocNumber"
    )]
    pub doc_number: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "PrivateNote"
    )]
    pub private_note: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Line {
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "Id")]
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "Description"
    )]
    pub description: String,
    #[serde(default, rename = "Amount")]
    pub amount: f32,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "DetailType"
    )]
    pub detail_type: String,
    #[serde(default, rename = "AccountBasedExpenseLineDetail")]
    pub account_based_expense_line_detail: AccountBasedExpenseLineDetail,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "LinkedTxn")]
    pub linked_txn: Vec<LinkedTxn>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AccountBasedExpenseLineDetail {
    #[serde(default, rename = "AccountRef")]
    pub account_ref: NtRef,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "BillableStatus"
    )]
    pub billable_status: String,
    #[serde(default, rename = "TaxCodeRef")]
    pub tax_code_ref: NtRef,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PurchaseEx {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub any: Vec<Any>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Any {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "declaredType"
    )]
    pub declared_type: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scope: String,
    #[serde(default)]
    pub value: NtRef,
    #[serde(default)]
    pub nil: bool,
    #[serde(default, rename = "globalScope")]
    pub global_scope: bool,
    #[serde(default, rename = "typeSubstituted")]
    pub type_substituted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "FileName")]
    pub file_name: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "FileAccessUri"
    )]
    pub file_access_uri: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "TempDownloadUri"
    )]
    pub temp_download_uri: String,
    #[serde(default, rename = "Size")]
    pub size: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub domain: String,
    #[serde(default)]
    pub sparse: bool,
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "Id")]
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "SyncToken"
    )]
    pub sync_token: String,
    #[serde(rename = "MetaData")]
    pub meta_data: MetaData,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "AttachableRef"
    )]
    pub attachable_ref: Vec<AttachableRef>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttachableRef {
    #[serde(default, rename = "EntityRef")]
    pub entity_ref: NtRef,
    #[serde(default, rename = "IncludeOnSend")]
    pub include_on_send: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillPayment {
    #[serde(default, rename = "VendorRef")]
    pub vendor_ref: NtRef,
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "PayType")]
    pub pay_type: String,
    #[serde(default, rename = "CheckPayment")]
    pub check_payment: Payment,
    #[serde(default, rename = "TotalAmt")]
    pub total_amt: f32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub domain: String,
    #[serde(default)]
    pub sparse: bool,
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "Id")]
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "SyncToken"
    )]
    pub sync_token: String,
    #[serde(rename = "MetaData")]
    pub meta_data: MetaData,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "DocNumber"
    )]
    pub doc_number: String,
    #[serde(rename = "TxnDate")]
    pub txn_date: NaiveDate,
    #[serde(default, rename = "CurrencyRef")]
    pub currency_ref: NtRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "Line")]
    pub line: Vec<Line>,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "PrivateNote"
    )]
    pub private_note: String,
    #[serde(default, rename = "CreditCardPayment")]
    pub credit_card_payment: Payment,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Payment {
    #[serde(default, rename = "CCAccountRef")]
    pub cc_account_ref: NtRef,
    #[serde(default, rename = "BankAccountRef")]
    pub bank_account_ref: NtRef,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        rename = "PrintStatus"
    )]
    pub print_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillResponse {
    #[serde(rename = "Bill")]
    pub bill: Bill,
    pub time: DateTime<Utc>,
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "APIError: status code -> {}, body -> {}",
            self.status_code, self.body
        )
    }
}
impl std::fmt::Debug for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "APIError: status code -> {}, body -> {}",
            self.status_code, self.body
        )
    }
}
impl std::error::Error for APIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

