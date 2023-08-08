use intuit_oxi_auth::Authorized;
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue},
    Method, Request,
};
use serde::Serialize;
use url::Url;

use crate::error::APIError;

use super::quickbooks::Quickbooks;

impl Quickbooks<Authorized> {
    pub(crate) fn build_url(
        &self,
        path: &str,
        query: &Option<&[(&str, &str)]>,
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

    pub(crate) fn build_headers(&self, header_type: &str) -> Result<HeaderMap, InvalidHeaderValue> {
        let bt = format!("Bearer {}", self.client.access_token().secret());
        let bearer =
            header::HeaderValue::from_str(&bt).expect("Invalid access token in Authorized Client");
        let mut headers = header::HeaderMap::new();
        headers.append(header::AUTHORIZATION, bearer);
        headers.append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(header_type)?,
        );
        headers.append(header::ACCEPT, header::HeaderValue::from_str(header_type)?);
        Ok(headers)
    }

    pub(crate) fn build_request<B>(&self, method: &Method, url: Url, headers: HeaderMap, body: &Option<B>) -> Result<Request, APIError>
    where 
        B: Serialize
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
        method: Method,
        path: &str,
        body: Option<B>,
        query: Option<&[(&str, &str)]>,
    ) -> super::quickbooks::Result<Request>
    where
        B: Serialize,
    {
        let url = self.build_url(path, &query)?;
        let headers = self.build_headers("application/json")?;
        let request = self.build_request(&method, url, headers, &body)?;

        log::info!(
            "Built Request with params: \n\t{}\n\t{}\n\t{}\n\t{:?}",
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
