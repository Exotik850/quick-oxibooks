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
use std::{fmt::Display, error::Error};
#[allow(dead_code)]
use std::sync::Arc;


use intuit_oauth::{AuthClient, AuthorizeType, Authorized, Unauthorized};
use reqwest::{header, Client, Method, Request, StatusCode, Url};
use serde::Serialize;

/// Endpoint for the QuickBooks API.
const ENDPOINT: &str = "https://sandbox-quickbooks.api.intuit.com/v3/";
// const ENDPOINT: &str = "https://quickbooks.api.intuit.com/v3/";


// #[derive(Debug)]
// pub struct APIError {
//     pub status_code: StatusCode,
//     pub body: String,
// }

// impl Error for APIError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         None
//     }
// }

// impl From<reqwest::Error> for APIError {
//     fn from(value: reqwest::Error) -> Self {
//         Self {
//             status_code: value.status().unwrap_or(StatusCode::EXPECTATION_FAILED),
//             body: value.to_string()
//         }
//     }
// }

// impl Display for APIError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write! (
//             f,
//             "APIError : Status Code: {} -> {}", 
//             self.status_code, self.body
//         )
//     }
// }

/// Entrypoint for interacting with the QuickBooks API.
#[derive(Debug, Clone)]
pub struct Quickbooks<T>
where
    T: AuthorizeType,
{
    redirect_uri: String,
    pub(crate) company_id: String,
    client: Arc<AuthClient<T>>,
    pub(crate) http_client: Arc<Client>,
}

impl Quickbooks<Unauthorized> {
    /// Create a new QuickBooks client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key your requests will work.
    pub async fn new<I, K, B, R>(
        client_id: I,
        client_secret: K,
        company_id: B,
        redirect_uri: R,
    ) -> Quickbooks<Authorized>
    where
        I: Display,
        K: Display,
        B: Display,
        R: Display,
    {
        let client = AuthClient::new(
            &client_id,
            &client_secret,
            &redirect_uri,
            &company_id,
            intuit_oauth::Environment::SANDBOX,
        )
        .await;

        let client = client.authorize().await;

        let qb = Quickbooks {
            company_id: company_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            client: Arc::new(client),
            http_client: Arc::new(Client::new()),
        };

        qb
    }

    /// Create a new QuickBooks client struct from environment variables. It
    /// takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key and your requests will work.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env<C: Display>(company_id: C) -> Quickbooks<Authorized> {
        let redirect_uri = dotenv::var("INTUIT_REDIRECT_URI").unwrap();
        let client =
            AuthClient::new_from_env(&company_id, intuit_oauth::Environment::SANDBOX).await;
        let mut client = client.authorize().await;
        client.refresh_access_token().await;

        Quickbooks {
            redirect_uri,
            company_id: company_id.to_string(),
            client: Arc::new(client),
            http_client: Arc::new(Client::new()),
        }
    }
}

impl Quickbooks<Authorized> {
    pub fn request<B>(
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

        let bt = format!("Bearer {}", self.client.get_tokens().0.secret());
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

        let mut rb = self
            .http_client
            .request(method.clone(), url)
            .headers(headers);

        if let Some(val) = query {
            rb = rb.query(&val);
            rb = rb.query(&[("minorversion", "65")])
        }

        // Add the body, this is to ensure our GET and DELETE calls succeed.
        if method != Method::GET && method != Method::DELETE {
            rb = rb.json(&body);
        }

        // Build the request.
        rb.build().unwrap()
    }

    // pub async fn get_invoice_by_doc_num(&self, doc_num: &str) -> Result<Invoice, APIError> {
        // let request = self.request(
        //     Method::GET,
        //     &format!("company/{}/query", self.company_id),
        //     (),
        //     Some(&[(
        //         "query",
        //         &format!(
        //             "select * from Invoice where DocNumber = '{doc_num}' MAXRESULTS {QUERY_PAGE_SIZE}"
        //         ),
        //     )]),
        // );

    //     let resp = self.http_client.execute(request).await.unwrap();
        // match resp.status() {
        //     StatusCode::OK => (),
        //     s => {
        //         return Err(APIError {
        //             status_code: s,
        //             body: resp.text().await.unwrap(),
        //         })
        //     }
        // };

    //     let r: QueryResponseExt<Invoice> = resp.json().await.unwrap();

    //     Ok(r.query_response.items[0].clone())
    // }
}

