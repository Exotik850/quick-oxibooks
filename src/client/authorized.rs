use intuit_oxi_auth::AuthError;
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue}, Method, Request
};
use serde::Serialize;
use url::Url;

use super::quickbooks::Quickbooks;
use crate::error::APIError;

impl Quickbooks {
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
        &self,
        content_type: &str,
    ) -> Result<HeaderMap, InvalidHeaderValue> {
        let bt = format!("Bearer {}", self.client.secret());
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
        B: Serialize,
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

    pub async fn request<B>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
        query: Option<&[(&str, &str)]>,
    ) -> super::quickbooks::Result<Request>
    where
        B: Serialize,
    {
        if self.client.is_expired() {
            self.client.refresh_access_token_async().await?;
        }

        let url = self.build_url(path, query)?;
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

    pub fn set_refresh_token(&self, refresh_token: String) -> Result<(), AuthError> {
        self.client.replace_tokens(refresh_token)
    }
}

#[cfg(feature = "cache")]
impl Quickbooks {
    pub async fn cleanup(&self) -> Result<(), AuthError> {
        self.client.cleanup_async(&self.key).await?;
        log::info!("Cleaned up Quickbooks client");
        Ok(())
    }
}
