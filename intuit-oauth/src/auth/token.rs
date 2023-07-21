use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    PRODUCTION,
    SANDBOX
}

impl Environment {
    pub fn discovery_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer.intuit.com/.well-known/openid_configuration/",
            Environment::SANDBOX => "https://developer.intuit.com/.well-known/openid_sandbox_configuration/",
        }
    }
    pub fn migration_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://developer-sandbox.api.intuit.com/v2/oauth2/tokens/migrate",
            Environment::SANDBOX => "https://developer.api.intuit.com/v2/oauth2/tokens/migrate",
        }
    }
    pub fn user_info_url(&self) -> &'static str {
        match self {
            Environment::PRODUCTION => "https://accounts.platform.intuit.com/v1/openid_connect/userinfo",
            Environment::SANDBOX => "https://sandbox-accounts.platform.intuit.com/v1/openid_connect/userinfo",
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub(crate) struct DiscoveryDoc {
    #[serde(default)]
    issuer: String,
    #[serde(default)]
    pub authorization_endpoint: String,
    #[serde(default)]
    pub token_endpoint: String,
    #[serde(default)]
    userinfo_endpoint: String,
    #[serde(default)]
    pub revocation_endpoint: String,
    #[serde(default)]
    jwks_uri: String,
    #[serde(default)]
    response_types_supported: Vec<String>,
    #[serde(default)]
    subject_types_supported: Vec<String>,
    #[serde(default)]
    id_token_signing_alg_values_supported: Vec<String>,
    #[serde(default)]
    scopes_supported: Vec<String>,
    #[serde(default)]
    token_endpoint_auth_methods_supported: Vec<String>,
    #[serde(default)]
    claims_supported: Vec<String>
}