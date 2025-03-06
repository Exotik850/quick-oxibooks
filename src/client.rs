use std::{
    ops::Deref,
    time::Duration,
};

use base64::Engine;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue},
    Client, Method, Request,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{error::APIError, limiter::RateLimiter, DiscoveryDoc, Environment};

// Rate Limit:
// Sandbox - 500 req / min
// Production - 500 req / min, 10 req / sec
// Batch - 30 req / batch & 40 batches / min
// Wait 60 seconds after throttle

const RATE_LIMIT: usize = 500;
const BATCH_RATE_LIMIT: usize = 40;
const RESET_DURATION: Duration = Duration::from_secs(60);

#[derive(Serialize)]
pub struct QBContext {
    pub(crate) environment: Environment,
    pub(crate) company_id: String,
    pub(crate) access_token: String,
    pub(crate) expires_in: DateTime<Utc>,
    // TODO Check if this should be in an option
    // pub(crate) refresh_token: Option<String>,
    pub(crate) discovery_doc: DiscoveryDoc,
    #[serde(skip)]
    pub(crate) qbo_limiter: RateLimiter,
    // #[serde(skip)]
    // pub(crate) batch_limiter: RateLimiter,
}

pub struct RefreshableQBContext {
    pub(crate) context: QBContext,
    pub(crate) refresh_token: String,
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
    /// Creates a new `QBContext` with the given parameters
    pub async fn new(
        environment: Environment,
        company_id: String,
        access_token: String,
        client: &Client,
    ) -> Result<Self, APIError> {
        Ok(Self {
            environment,
            company_id,
            access_token,
            expires_in: Utc::now() + chrono::Duration::hours(999),
            discovery_doc: DiscoveryDoc::get_async(environment, client).await?,
            qbo_limiter: RateLimiter::new(RATE_LIMIT, RESET_DURATION),
            // batch_limiter: RateLimiter::new(BATCH_RATE_LIMIT, RESET_DURATION),
        })
    }

    pub async fn new_from_env(environment: Environment, client: &Client) -> Result<Self, APIError> {
        let company_id = std::env::var("QB_COMPANY_ID")?;
        let access_token = std::env::var("QB_ACCESS_TOKEN")?;
        let context = Self::new(environment, company_id, access_token, client).await?;
        Ok(context)
    }

    #[must_use] pub fn with_refresh(self, refresh_token: String) -> RefreshableQBContext {
        RefreshableQBContext {
            context: self,
            refresh_token,
        }
    }

    #[must_use] pub fn with_access_token(self, access_token: String) -> Self {
        Self { access_token, ..self }
    }

    /// Checks if the current context is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_in
    }

    pub async fn check_authorized(&self, client: &Client) -> Result<bool, APIError> {
        let request = client
            .request(Method::GET, self.environment.user_info_url())
            .bearer_auth(&self.access_token)
            .header("ACCEPT", "application/json")
            .build()?;
        let response = client.execute(request).await?;
        let status = response.status();
        if !status.is_success() {
            println!(
                "Failed to check authorized status: {} - {}",
                status,
                response.text().await?
            );
        }
        Ok(status.is_success())
    }
}

impl RefreshableQBContext {
    pub async fn refresh_access_token(
        &mut self,
        client_id: &str,
        client_secret: &str,
        client: &Client,
    ) -> Result<(), APIError> {
        let auth_string = format!("{client_id}:{client_secret}");
        let auth_string = base64::engine::general_purpose::STANDARD.encode(auth_string);

        let request = client
            .request(Method::POST, &self.context.discovery_doc.token_endpoint)
            .bearer_auth(auth_string)
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=refresh_token&refresh_token={}",
                &self.refresh_token
            ))
            .build()?;

        let response = client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.json().await?));
        }

        let AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in,
            ..
        } = response.json().await?;

        self.refresh_token = refresh_token;
        self.context.access_token = access_token;
        self.context.expires_in = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        Ok(())
    }
}

impl Deref for RefreshableQBContext {
    type Target = QBContext;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

pub(crate) fn build_headers(
    content_type: &str,
    access_token: &str,
) -> Result<HeaderMap, InvalidHeaderValue> {
    let bt = format!("Bearer {access_token}");
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
            .extend_pairs(q)
            .extend_pairs([("minorVersion", "65")]);
    }
    Ok(url)
}
