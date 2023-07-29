/*!
 * A rust library for interacting with the QuickBooks API.
 *
 * For more information, you can check out their documentation at:
 * https://developer.intuit.com/app/developer/qbo/docs/develop
 *
 * ORIGINIALLY FROM https://github.com/oxidecomputer/cio
 * LICENSED UNDER APACHE 2.0
 *
 */
#[allow(dead_code)]
use std::fmt::Display;
use std::sync::Arc;

use crate::error::APIError;
use intuit_oauth::{AuthClient, AuthorizeType, Authorized, Environment, Unauthorized};
use reqwest::{header, Client, Method, Request, Url};
use serde::Serialize;

type Result<T> = std::result::Result<T, APIError>;

/// Entrypoint for interacting with the QuickBooks API.
#[derive(Debug, Clone)]
pub struct Quickbooks<T>
where
    T: AuthorizeType,
{
    pub(crate) company_id: String,
    pub environment: Environment,
    client: Arc<AuthClient<T>>,
    pub(crate) http_client: Arc<Client>,
}

impl Quickbooks<Unauthorized> {
    /// Create a new QuickBooks client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key your requests will work.
    #[allow(unused)]
    pub async fn new<I, K, B, R>(
        client_id: I,
        client_secret: K,
        company_id: B,
        redirect_uri: R,
        environment: Environment,
    ) -> Result<Quickbooks<Authorized>>
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
            environment,
        )
        .await?;

        let client = client.authorize().await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client: Arc::new(client),
            environment,
            http_client: Arc::new(Client::new()),
        })
    }

    /// Create a new QuickBooks client struct from environment variables. It
    /// takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key and your requests will work.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env<C: Display>(
        company_id: C,
        environment: Environment,
    ) -> Result<Quickbooks<Authorized>> {
        let client = AuthClient::new_from_env(&company_id, environment).await?;
        let mut client = client.authorize().await?;
        client.refresh_access_token().await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client: Arc::new(client),
            environment,
            http_client: Arc::new(Client::new()),
        })
    }
}

impl Quickbooks<Authorized> {
    pub fn request<B>(
        &self,
        method: Method,
        path: &str,
        body: B,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Request>
    where
        B: Serialize,
    {
        let base = Url::parse(self.environment.endpoint_url()).unwrap();
        let url = base.join(path)?;
        let bt = format!("Bearer {}", self.client.access_token().secret());
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

        Ok(rb.build()?)
    }
}
