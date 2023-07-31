use intuit_oxi_auth::Authorized;
use reqwest::{header, Request, Method};
use serde::Serialize;
use url::Url;

use super::quickbooks::Quickbooks;





impl Quickbooks<Authorized> {
    pub fn request<B>(
        &self,
        method: Method,
        path: &str,
        body: B,
        query: Option<&[(&str, &str)]>,
    ) -> super::quickbooks::Result<Request>
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

impl super::quickbooks::QBData<Authorized> for Quickbooks<Authorized> {
    fn get_data(&self) -> Option<&Authorized> {
        Some(&self.client.data)
    }
}