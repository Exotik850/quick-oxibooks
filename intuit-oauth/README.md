# intuit-oauth
This crate is a simple wrapper over the authorization process for Intuit applications

## Installation
Add this to your `Cargo.toml`
```toml
[dependencies]
intuit-oauth = "0.1.1"
```

# Basic Usage 
First create a new instance of the AuthClient

```rust
let client = AuthClient::new(
    CLIENT_ID, 
    CLIENT_SECRET,
    REDIRECT_URI,
    REALM_ID, // "Company ID"
    Environment::Sandbox
).await;
// The parameters need only implement ToString from display, Vec<u8>, &str, etc

/// new_from_env is basically the same, just grabbing the Client ID, Secret, and Redirect Url from environment variables "INTUIT_CLIENT_ID, INTUIT_..." etc
```

Authorize the client either by chaining or seperate call
```rust
let client = AuthClient::new(
    CLIENT_ID, 
    CLIENT_SECRET,
    REDIRECT_URI,
    REALM_ID, // "Company ID"
    Environment::Sandbox
).await
.authorize()
.await
.unwrap();
```

or 
```rust
let client = ...
let authorized = client.authorize().await.unwrap();
```

Refresh the access token with the refresh token periodically to avoid expiration

```rust
auth_client.refresh_access_token().await.unwrap();
```

## Contributing
Pull requests are always welcome! Please open an issue first to discuss any major changes.

## License
Licensed under MIT License 2023