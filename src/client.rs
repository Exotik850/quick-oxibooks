use reqwest::{
    header::{self, HeaderMap, InvalidHeaderValue}, Client, Method, Request
};
use url::Url;

use crate::{error::APIError, Environment};

pub struct QBContext {
    pub(crate) client: Client,
    pub environment: Environment,
    pub(crate) company_id: String,
    pub(crate) access_token: String,
}

impl QBContext {
    pub fn new(environment: Environment, company_id: String, access_token: String) -> Self {
        Self {
            client: Client::new(),
            environment,
            company_id,
            access_token,
        }
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
