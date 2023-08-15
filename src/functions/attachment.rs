use super::QBResponse;
use crate::{client::Quickbooks, error::APIError};
use async_trait::async_trait;
use intuit_oxi_auth::Authorized;
use quickbooks_types::{Attachable, QBAttachable, QBItem};
use reqwest::Method;
use reqwest::header::{self, HeaderName, HeaderValue};
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait QBAttachment: QBItem + QBAttachable {
    async fn upload(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError>;
}

#[async_trait]
impl QBAttachment for Attachable {
    async fn upload(&self, qb: &Quickbooks<Authorized>) -> Result<Self, APIError> {
        if !self.can_upload() {
            return Err(APIError::AttachableUploadMissingItems);
        }

        let file_name = self.file_name.as_ref().unwrap();

        let path = format!("company/{}/upload", qb.company_id);
        let url = qb.build_url(&path, &Some(&[]))?;
        let headers = qb
            .build_headers("multipart/form-data")
            .await?;

        // let json_headers = {
        //     let mut headers = header::HeaderMap::new();
        //     headers.append(
        //         HeaderName::from_static("Content-Transfer-Encoding"),
        //         HeaderValue::from_static("8bit")
        //     );
        //     headers
        // };

        let json_part =
            Part::text(serde_json::to_string(self).expect("Couldn't Serialize Attachment"))
                .mime_str("application/json; charset=UTF-8")?
                .file_name("attachment.json")
                // .headers(json_headers);
                ;

        let mut buf: Vec<u8> = vec![];
        let mut file = tokio::fs::File::open(&file_name).await?;
        file.read_to_end(&mut buf).await?;
        let encoded = base64::encode(buf);

        // let file_headers = {
        //     let mut headers = header::HeaderMap::new();
        //     headers.append(
        //         HeaderName::from_static("Content-Transfer-Encoding"),
        //         HeaderValue::from_static("base64")
        //     );
        //     headers
        // };

        let file_part = Part::bytes(encoded.into_bytes())
            .mime_str("image/jpg")?
            .file_name(file_name.to_string())
            // .headers(file_headers);
            ;

        let multipart = Form::new()
            .part("file_metadata_01", json_part)
            .part("file_content_01", file_part);

        let request = qb.http_client.request(Method::POST, url)
            // .headers(headers)
            .multipart(multipart)
            .build()?;

        let response = qb.http_client.execute(request).await?;

        let qb_response: QBResponse<Self> = response.json().await?;

        Ok(qb_response.object)
    }
}
