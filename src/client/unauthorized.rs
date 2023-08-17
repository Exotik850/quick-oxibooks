use std::sync::Arc;

use intuit_oxi_auth::{AuthClient, Authorized, Environment, Unauthorized};
use reqwest::Client;

use super::quickbooks::Quickbooks;

impl Quickbooks<Unauthorized> {
    /// Create a new QuickBooks client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key your requests will work.
    #[allow(unused)]
    pub async fn new(
        client_id: &str,
        client_secret: &str,
        company_id: &str,
        redirect_uri: &str,
        environment: Environment,
        #[cfg(feature = "cache")] key: &str,
    ) -> super::quickbooks::Result<Quickbooks<Authorized>> {
        let client = AuthClient::new(
            client_id,
            client_secret,
            redirect_uri,
            company_id,
            environment,
        )
        .await?;

        #[cfg(feature = "cache")]
        let client = client.authorize(None, key).await?;
        #[cfg(not(feature = "cache"))]
        let client = client.authorize(None).await?;

        log::info!("Authorized Quickbooks Client in {:?}", environment);

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client,
            environment,
            http_client: Arc::new(Client::new()),
            #[cfg(feature = "cache")]
            key: key.to_string(),
        })
    }

    /// Create a new QuickBooks client struct from environment variables.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env(
        company_id: &str,
        environment: Environment,
        #[cfg(feature = "cache")] key: &str,
    ) -> super::quickbooks::Result<Quickbooks<Authorized>> {
        let client = AuthClient::new_from_env(company_id, environment).await?;

        #[cfg(feature = "cache")]
        let client = client.authorize(None, key).await?;
        #[cfg(not(feature = "cache"))]
        let client = client.authorize(None).await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client,
            environment,
            http_client: Arc::new(Client::new()),
            #[cfg(feature = "cache")]
            key: key.to_string(),
        })
    }
}
