use std::sync::Arc;

use intuit_oxi_auth::{AuthClient, Environment, TokenSession};
use reqwest::Client;

use super::quickbooks::Quickbooks;

impl Quickbooks {
    /// Create a new `QuickBooks` client struct. It takes a type that can convert into
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
    ) -> super::quickbooks::Result<Quickbooks> {
        let client =
            AuthClient::new_async(client_id, client_secret, redirect_uri, environment).await?;

        #[cfg(feature = "cache")]
        let client = client.authorize_async(None, key).await?;
        #[cfg(not(feature = "cache"))]
        let client = client.authorize_async(None).await?;

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

    /// Create a new `QuickBooks` client struct from environment variables.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env(
        company_id: &str,
        environment: Environment,
        #[cfg(feature = "cache")] key: &str,
    ) -> super::quickbooks::Result<Quickbooks> {
        let client = AuthClient::new_from_env_async(environment).await?;

        #[cfg(feature = "cache")]
        let client = client.authorize_async(None, key).await?;
        #[cfg(not(feature = "cache"))]
        let client = client.authorize_async(None).await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client,
            environment,
            http_client: Arc::new(Client::new()),
            #[cfg(feature = "cache")]
            key: key.to_string(),
        })
    }

    pub async fn new_from_token(
        refresh_token: String,
        company_id: &str,
        environment: Environment,
        #[cfg(feature = "cache")] key: &str,
    ) -> super::quickbooks::Result<Self> {
        let client = AuthClient::new_from_token_async(refresh_token, environment).await?;
        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client,
            environment,
            http_client: Arc::new(Client::new()),
            #[cfg(feature = "cache")]
            key: key.to_string(),
        })
    }

    pub async fn new_from_session(
        session: TokenSession,
        company_id: &str,
        environment: Environment,
        #[cfg(feature = "cache")] key: &str,
    ) -> super::quickbooks::Result<Self> {
        let client = AuthClient::new_from_session_async(session, environment).await?;
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
