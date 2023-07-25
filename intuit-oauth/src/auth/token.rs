use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Environment {
    PRODUCTION,
    #[default] SANDBOX
}

impl Environment {
    #[inline]
    pub fn discovery_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer.intuit.com/.well-known/openid_configuration/",
            Environment::SANDBOX => "https://developer.intuit.com/.well-known/openid_sandbox_configuration/",
        }
    }

    #[inline]
    pub fn migration_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer-sandbox.api.intuit.com/v2/oauth2/tokens/migrate",
            Environment::SANDBOX => "https://developer.api.intuit.com/v2/oauth2/tokens/migrate",
        }
    }

    #[inline]
    pub fn user_info_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://accounts.platform.intuit.com/v1/openid_connect/userinfo",
            Environment::SANDBOX => "https://sandbox-accounts.platform.intuit.com/v1/openid_connect/userinfo",
        }
    }

    #[inline]
    pub fn endpoint_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://quickbooks.api.intuit.com/v3/",
            Environment::SANDBOX => "https://sandbox-quickbooks.api.intuit.com/v3/",
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub(crate) struct DiscoveryDoc {
    issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    userinfo_endpoint: String,
    pub revocation_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
    scopes_supported: Vec<String>,
    token_endpoint_auth_methods_supported: Vec<String>,
    claims_supported: Vec<String>
}