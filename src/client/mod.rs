use crate::{APIResult, Environment};
use serde::Serialize;
use ureq::{
    http::{request::Builder, Method, Request},
    SendBody,
};
mod context;
mod refresh;
pub use context::QBContext;
pub use refresh::RefreshableQBContext;
use urlencoding::encode;

pub(crate) fn set_headers(content_type: &str, access_token: &str, request: Builder) -> Builder {
    let bt = format!("Bearer {access_token}");
    let mut request = request.header("Authorization", bt);
    if content_type != "multipart/form-data" {
        request = request.header("Content-Type", content_type);
    }
    request.header("Accept", "application/json")
}

pub(crate) fn build_request<B, S, SS>(
    method: Method,
    path: &str,
    body: Option<&B>,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
    content_type: &str,
    environment: Environment,
    access_token: &str,
) -> APIResult<Request<SendBody<'static>>>
where
    B: Serialize,
    S: AsRef<str>,
    SS: AsRef<str>,
{
    let url = build_url(environment, path, query);
    let mut request = Request::builder().method(method.clone()).uri(url.as_str());
    request = set_headers(content_type, access_token, request);

    let request = match (method == Method::GET || method == Method::DELETE, body) {
        (true, _) => request.body(SendBody::none()),
        (false, Some(body)) => {
            let json_bytes = serde_json::to_vec(body)?;
            let reader = std::io::Cursor::new(json_bytes);
            request.body(SendBody::from_owned_reader(reader))
        }
        (false, None) => request.body(SendBody::none()),
    }?;

    #[cfg(feature = "logging")]
    log::debug!(
        "Built Request with params: {}-{}-{}",
        path,
        method,
        if body.is_some() {
            "With JSON Body"
        } else {
            "No JSON Body"
        },
    );

    Ok(request)
}

pub(crate) fn build_url<'a, S, SS>(
    environment: Environment,
    path: &str,
    query: Option<impl IntoIterator<Item = (S, SS)>>,
) -> String
where
    S: AsRef<str>,
    SS: AsRef<str>,
{
    // let url = Url::parse(environment.endpoint_url())?;
    let mut url = environment.endpoint_url().to_string();
    url.push_str(path);
    if let Some(q) = query {
        let query_string: String = q
            .into_iter()
            .map(|(k, v)| {
                (
                    encode(k.as_ref()).to_string(),
                    encode(v.as_ref()).to_string(),
                )
            })
            .map(|(k, v)| format!("{k}={v}"))
            .chain(std::iter::once("minorversion=75".to_string()))
            .collect::<Vec<_>>()
            .join("&");
        if !query_string.is_empty() {
            url.push('?');
            url.push_str(&query_string);
        }
    }
    url
}
