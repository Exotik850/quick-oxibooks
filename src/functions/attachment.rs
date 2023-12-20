use std::path::{Path, PathBuf};

use async_trait::async_trait;
use base64::Engine;
use quickbooks_types::{content_type_from_ext, Attachable, QBAttachable, QBItem};
use reqwest::{
    header::{self, HeaderValue}, multipart::{Form, Part}, Method, Request
};

use crate::{client::Quickbooks, error::APIError};

#[async_trait]
pub trait QBAttachment: QBItem + QBAttachable {
    async fn upload(&self, qb: &Quickbooks, access_token: &str) -> Result<Self, APIError>;
    async fn make_upload_request(
        &self,
        qb: &Quickbooks,
        access_token: &str,
    ) -> Result<Request, APIError>;
}

async fn _make_file_part(file_name: impl AsRef<Path>) -> Result<Part, APIError> {
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
    let ext: PathBuf = file_name.as_ref().to_path_buf();
    let ct = content_type_from_ext(ext.extension().unwrap().to_str().unwrap());

    let file_part = Part::bytes(encoded.into_bytes())
        .mime_str(ct)?
        .file_name(
            file_name
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
        .headers(file_headers);

    Ok(file_part)
}

#[async_trait]
impl QBAttachment for Attachable {
    async fn upload(&self, qb: &Quickbooks, access_token: &str) -> Result<Self, APIError> {
        if !self.can_upload() {
            return Err(APIError::AttachableUploadMissingItems);
        }

        let request = self.make_upload_request(qb, access_token).await?;

        let response = qb.http_client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
        }

        let mut qb_response: AttachableResponseExt = response.json().await?;
        if qb_response.ar.is_empty() {
            return Err(APIError::NoAttachableObjects);
        };

        let obj = qb_response.ar.swap_remove(0).attachable;

        log::info!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

        Ok(obj)
    }

    async fn make_upload_request(
        &self,
        qb: &Quickbooks,
        access_token: &str,
    ) -> Result<Request, APIError> {
        let file_name = self
            .file_name
            .as_ref()
            .ok_or(APIError::AttachableUploadMissingItems)?;

        let path = format!("company/{}/upload", qb.company_id);
        let url = qb.build_url(&path, Some(&[]))?;
        let request_headers = Quickbooks::build_headers("application/pdf", access_token)?;

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
    #[allow(dead_code)]
    time: String,
}

#[derive(serde::Deserialize, Debug)]
struct AttachableResponse {
    #[serde(rename = "Attachable")]
    attachable: Attachable,
}
