use chrono::{DateTime, Utc};
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue}, Client, Method, Request
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{error::APIError, DiscoveryDoc, Environment};

#[derive(Serialize, Deserialize)]
pub struct QBContext {
    pub environment: Environment,
    pub company_id: String,
    pub access_token: String,
    pub expires_in: DateTime<Utc>,
    // TODO Check if this should be in an option
    pub refresh_token: Option<String>,
    pub discovery_doc: DiscoveryDoc,
}

#[derive(Serialize, Deserialize)]
struct AuthTokenResponse {
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    x_refresh_token_expires_in: u64,
    access_token: String,
}

impl QBContext {
    /// Checks if the current context is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_in
    }

    /// Refreshes the access_token, does not check if it's expired before it does so
    pub async fn refresh_access_token(&mut self, client: &Client) -> Result<(), APIError> {
        // TODO Use types to prevent this from happening
        let Some(refresh_token) = self.refresh_token.as_deref() else {
            return Err(APIError::NoRefreshToken);
        };

        let request = client
            .request(Method::POST, &self.discovery_doc.token_endpoint)
            .bearer_auth(&self.access_token)
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=refresh_token&refresh_token={}",
                refresh_token
            ))
            .build()?;

        let response = client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
        }

        let AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in,
            ..
        } = response.json().await?;

        self.refresh_token = Some(refresh_token);
        self.access_token = access_token;
        self.expires_in = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        Ok(())
    }
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

pub(crate) fn build_request<B: serde::Serialize>(
    method: Method,
    path: &str,
    body: Option<B>,
    query: Option<&[(&str, &str)]>,
    content_type: &str,
    environment: Environment,
    client: &Client,
    access_token: &str,
) -> Result<Request, APIError> {
    let url = build_url(environment, path, query)?;

    let headers = build_headers(content_type, access_token)?;

    let mut request = client.request(method.clone(), url).headers(headers);

    if method != Method::GET && method != Method::DELETE {
        request = request.json(&body);
    }

    let request = request.build()?;

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

pub(crate) fn build_url(
    environment: Environment,
    path: &str,
    query: Option<&[(&str, &str)]>,
) -> Result<Url, APIError> {
    let url = Url::parse(environment.endpoint_url())?;
    let mut url = url.join(path)?;
    if let Some(q) = query {
        url.query_pairs_mut()
            .extend_pairs(q.iter())
            .extend_pairs([("minorVersion", "65")]);
    }
    Ok(url)
}
