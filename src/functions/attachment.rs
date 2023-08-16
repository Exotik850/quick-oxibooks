use crate::{client::Quickbooks, error::APIError};
use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, Utc};
use intuit_oxi_auth::Authorized;
use quickbooks_types::{content_type_from_ext, Attachable, QBAttachable, QBItem};
use reqwest::header::{self, HeaderValue};
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::{Method, Request};
use std::path::PathBuf;

#[async_trait]
pub trait QBAttachment: QBItem + QBAttachable {
    async fn upload(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError>;
    async fn make_upload_request(&self, qb: &Quickbooks<Authorized>) -> Result<Request, APIError>;
}

async fn _make_file_part(file_name: &str) -> Result<Part, APIError> {
    let buf = tokio::fs::read(&file_name).await?;
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf);

    let file_headers = {
        let mut headers = header::HeaderMap::new();
        headers.append(
            "Content-Transfer-Encoding",
            HeaderValue::from_static("base64"),
        );
        headers
    };

    // Would've returned an error already if it was directory, safe to unwrap
    let ext: &PathBuf = &file_name.into();
    let ct = content_type_from_ext(&ext.extension().unwrap().to_string_lossy());

    let file_part = Part::bytes(encoded.into_bytes())
        .mime_str(ct)?
        .file_name(file_name.to_string())
        .headers(file_headers);

    Ok(file_part)
}

#[async_trait]
impl QBAttachment for Attachable {
    async fn upload(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        if !self.can_upload() {
            return Err(APIError::AttachableUploadMissingItems);
        }

        let request = self.make_upload_request(&qb).await?;

        let response = qb.http_client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
        }

        let mut qb_response: AttachableResponseExt = response.json().await?;
        let obj = qb_response.ar.remove(0).attachable;

        log::info!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

        Ok(obj)
    }

    async fn make_upload_request(&self, qb: &Quickbooks<Authorized>) -> Result<Request, APIError> {
        let file_name = self.file_name.as_ref().unwrap();

        let path = format!("company/{}/upload", qb.company_id);
        let url = qb.build_url(&path, &Some(&[]))?;
        let request_headers = qb.build_headers("multipart/form-data").await?;

        let json_body = serde_json::to_string(self).expect("Couldn't Serialize Attachment");
        let json_part = Part::text(json_body).mime_str("application/json")?;

        let file_part = _make_file_part(file_name).await?;

        let multipart = Form::new()
            .part("file_metadata_01", json_part)
            .part("file_content_01", file_part);

        Ok(qb
            .http_client
            .request(Method::POST, url)
            .headers(request_headers)
            .multipart(multipart)
            .build()?)
    }
}

#[derive(Debug, serde::Deserialize)]
struct AttachableResponseExt {
    #[serde(rename = "AttachableResponse")]
    ar: Vec<AttachableResponse>,
    time: DateTime<Utc>,
}

#[derive(serde::Deserialize, Debug)]
struct AttachableResponse {
    #[serde(rename = "Attachable")]
    attachable: Attachable,
}
