use http_client::http_types::{Method, Request};
use serde::Serialize;
use url::Url;

use crate::{error::APIError, Environment};
mod context;
mod refresh;
pub use context::QBContext;

pub(crate) fn set_headers(content_type: &str, access_token: &str, request: &mut Request) {
    let bt = format!("Bearer {access_token}");
    request.insert_header("Authorization", bt);
    if content_type != "multipart/form-data" {
        request.insert_header("Content-Type", content_type);
    }
    request.insert_header("Accept", "application/json");
}

pub(crate) fn build_request<B>(
    method: Method,
    path: &str,
    body: Option<&B>,
    query: Option<&[(&str, &str)]>,
    content_type: &str,
    environment: Environment,
    access_token: &str,
) -> Result<Request, APIError>
where
    B: Serialize,
{
    let url = build_url(environment, path, query)?;
    let mut request = Request::new(method, url);
    set_headers(content_type, access_token, &mut request);

    if method != Method::Get && method != Method::Delete {
        if let Some(body) = body {
            let value = serde_json::to_string(body)?;
            request.set_body(value);
        }
    }

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
