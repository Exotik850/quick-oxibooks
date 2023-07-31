use std::{fmt::Display, sync::Arc};

use intuit_oxi_auth::{Unauthorized, Environment, Authorized, AuthClient};
use reqwest::Client;

use super::quickbooks::Quickbooks;


impl Quickbooks<Unauthorized> {
    /// Create a new QuickBooks client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key your requests will work.
    #[allow(unused)]
    pub async fn new<I, K, B, R>(
        client_id: I,
        client_secret: K,
        company_id: B,
        redirect_uri: R,
        environment: Environment,
    ) -> super::quickbooks::Result<Quickbooks<Authorized>>
    where
        I: Display,
        K: Display,
        B: Display,
        R: Display,
    {
        let client = AuthClient::new(
            &client_id,
            &client_secret,
            &redirect_uri,
            &company_id,
            environment,
        )
        .await?;

        let client = client.authorize(None).await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client: Arc::new(client),
            environment,
            http_client: Arc::new(Client::new()),
        })
    }

    /// Create a new QuickBooks client struct from environment variables. It
    /// takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API key and your requests will work.
    /// We pass in the token and refresh token to the client so if you are storing
    /// it in a database, you can get it first.
    pub async fn new_from_env<C: Display>(
        company_id: C,
        environment: Environment,
    ) -> super::quickbooks::Result<Quickbooks<Authorized>> {
        let client = AuthClient::new_from_env(&company_id, environment).await?
            .authorize(None).await?;
        // client.refresh_access_token().await?;

        Ok(Quickbooks {
            company_id: company_id.to_string(),
            client: Arc::new(client),
            environment,
            http_client: Arc::new(Client::new()),
        })
    }
}

impl super::quickbooks::QBData<Unauthorized> for Quickbooks<Unauthorized> {
    fn get_data(&self) -> Option<&Unauthorized> {
        None
    }
}