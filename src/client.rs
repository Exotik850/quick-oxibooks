use std::sync::Arc;

use intuit_oxi_auth::Environment;
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue}, Client, Method, Request
};
use url::Url;

use crate::error::APIError;

/// Entrypoint for interacting with the `QuickBooks` API.
#[derive(Debug)]
pub struct Quickbooks {
    pub(crate) company_id: String,
    pub environment: Environment,
    pub(crate) http_client: Arc<Client>,
}

impl Quickbooks {
    pub fn new(company_id: String, environment: Environment) -> Self {
        Self {
            company_id,
            environment,
            http_client: Default::default(),
        }
    }

    pub(crate) fn build_url(
        &self,
        path: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Url, url::ParseError> {
        let base = Url::parse(self.environment.endpoint_url())?;
        let mut url = base.join(path)?;
        if let Some(q) = query {
            url.query_pairs_mut()
                .extend_pairs(q.iter())
                .extend_pairs([("minorVersion", "65")]);
        }
        Ok(url)
    }

    pub(crate) fn build_headers(
        content_type: &str,
        access_token: &str,
    ) -> Result<HeaderMap, InvalidHeaderValue> {
        let bt = format!("Bearer {}", access_token);
        let bearer =
            header::HeaderValue::from_str(&bt).expect("Invalid access token in Authorized Client");
        let mut headers = header::HeaderMap::new();
        headers.append(header::AUTHORIZATION, bearer);
        if content_type != "multipart/form-data" {
            headers.append(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(content_type)?,
            );
        }
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_str("application/json")?,
        );
        Ok(headers)
    }

    pub(crate) fn build_request<B>(
        &self,
        method: &Method,
        url: Url,
        headers: HeaderMap,
        body: &Option<B>,
    ) -> Result<Request, APIError>
    where
        B: serde::Serialize,
    {
        let mut rb = self
            .http_client
            .request(method.clone(), url)
            .headers(headers);

        // Add the body, this is to ensure our GET and DELETE calls succeed.
        if method != Method::GET && method != Method::DELETE {
            rb = rb.json(body);
        }

        Ok(rb.build()?)
    }

    pub fn request<B>(
        &self,
        access_token: &str,
        method: Method,
        path: &str,
        body: Option<B>,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Request, APIError>
    where
        B: serde::Serialize,
    {
        // if self.client.is_expired() {
        //     self.client.refresh_access_token_async().await?;
        // }

        let url = self.build_url(path, query)?;
        let headers = Self::build_headers("application/json", access_token)?;
        let request = self.build_request(&method, url, headers, &body)?;

        log::info!(
            "Built Request with params: {}-{}-{}-{:?}",
            path,
            method,
            if body.is_some() {
                "With JSON Body"
            } else {
                "No JSON Body"
            },
            query
        );

        Ok(request)
    }
}
