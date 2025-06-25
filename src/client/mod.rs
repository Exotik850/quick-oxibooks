use serde::Serialize;
use ureq::http::{request::Builder, Method, Request};
use url::Url;

use crate::{APIResult, Environment};
mod context;
mod refresh;
pub use context::QBContext;
pub use refresh::RefreshableQBContext;

pub(crate) fn set_headers(content_type: &str, access_token: &str, request: Builder) -> Builder {
    let bt = format!("Bearer {access_token}");
    let mut request = request.header("Authorization", bt);
    if content_type != "multipart/form-data" {
        request = request.header("Content-Type", content_type);
    }
    request.header("Accept", "application/json")
}

pub(crate) fn build_request<B>(
    method: Method,
    path: &str,
    body: Option<&B>,
    query: Option<&[(&str, &str)]>,
    content_type: &str,
    environment: Environment,
    access_token: &str,
) -> APIResult<Request<String>>
where
    B: Serialize,
{
    let url = build_url(environment, path, query)?;
    let mut request = Request::builder().method(method.clone()).uri(url.as_str());
    request = set_headers(content_type, access_token, request);

    let request = if method != Method::GET && method != Method::DELETE {
        if let Some(body) = body {
            let value = serde_json::to_string(body)?;
            request.body(value)
        } else {
            request.body(String::new())
        }
    } else {
        request.body(String::new())
    }?;

    log::debug!(
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
) -> Result<Url, url::ParseError> {
    let url = Url::parse(environment.endpoint_url())?;
    let mut url = url.join(path)?;
    if let Some(q) = query {
        url.query_pairs_mut()
            .extend_pairs(q)
            .extend_pairs([("minorVersion", "65")]);
    }
    Ok(url)
}
