use base64::Engine;
use http_client::{http_types::Method, HttpClient, Request};
use serde::{Deserialize, Serialize};

use crate::error::APIError;

use super::QBContext;

pub struct RefreshableQBContext {
    pub(crate) context: QBContext,
    pub(crate) refresh_token: String,
}

impl RefreshableQBContext {
    /// Refreshes the access token using the refresh token
    pub async fn refresh_access_token<Client: HttpClient>(
        &mut self,
        client_id: &str,
        client_secret: &str,
        client: &Client,
    ) -> Result<(), APIError> {
        let auth_string = format!("{client_id}:{client_secret}");
        let auth_string = base64::engine::general_purpose::STANDARD.encode(auth_string);

        // let request = client
        //     .request(Method::POST, &self.context.discovery_doc.token_endpoint)
        //     .bearer_auth(auth_string)
        //     .header("ACCEPT", "application/json")
        //     .header("Content-Type", "application/x-www-form-urlencoded")
        //     .body(format!(
        //         "grant_type=refresh_token&refresh_token={}",
        //         &self.refresh_token
        //     ))
        //     .build()?;

        let mut request = Request::new(
            Method::Post,
            self.context.discovery_doc.token_endpoint.as_str(),
        );
        request.insert_header("Authorization", format!("Basic {auth_string}"));
        request.insert_header("Content-Type", "application/x-www-form-urlencoded");
        request.insert_header("Accept", "application/json");
        request.set_body(format!(
            "grant_type=refresh_token&refresh_token={}",
            &self.refresh_token
        ));

        let response = client.send(request).await?;

        if !response.status().is_success() {
            return Err(APIError::InvalidClient);
        }

        let mut response = response;
        let AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in,
            ..
        } = response.body_json().await?;

        self.refresh_token = refresh_token;
        self.context.access_token = access_token;
        self.context.expires_in = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct AuthTokenResponse {
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    x_refresh_token_expires_in: u64,
    access_token: String,
}

impl std::ops::Deref for RefreshableQBContext {
    type Target = QBContext;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
