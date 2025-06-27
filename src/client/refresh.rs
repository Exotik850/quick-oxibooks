use base64::Engine;
use serde::{Deserialize, Serialize};
use ureq::{http::Request, Agent};

use super::QBContext;
use crate::error::{APIError, APIErrorInner};

pub struct RefreshableQBContext {
    pub(crate) context: QBContext,
    pub(crate) refresh_token: String,
}

impl RefreshableQBContext {
    /// Refreshes the access token using the refresh token
    pub fn refresh_access_token(
        &mut self,
        client_id: &str,
        client_secret: &str,
        client: &Agent,
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

        let request = Request::post(self.context.discovery_doc.token_endpoint.as_str())
            .header("Authorization", format!("Basic {auth_string}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(format!(
                "grant_type=refresh_token&refresh_token={}",
                &self.refresh_token
            ))?;

        let response = client.run(request)?;

        if !response.status().is_success() {
            return Err(APIErrorInner::InvalidClient.into());
        }

        let AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in,
            ..
        } = response.into_body().read_json()?;

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
