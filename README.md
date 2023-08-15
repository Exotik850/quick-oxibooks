# quick-oxibooks
Quickbooks + Rust (oxi)

## Features:
 - Simple to use API for querying, sending, creating, and more with Quickbooks Online API
 - Fully featured types with all the fields for your accounting needs
 - Async runtime by default (blocking client soon)
 - Error handling from `thiserror`

Very early in development, expect bugs and issues and changes

# Getting Started
Add this to your `Cargo.toml`
```toml
[dependencies]
quick-oxibooks = "0.1.1"
```

# Basic Usage
Create a Quickbooks client object either by putting the Client ID, Secret, Refresh URL directly into the constructor or grabbing from the environment variables `INTUIT_CLIENT_ID`, `INTUIT_CLIENT_SECRET`, and `INTUIT_REDIRECT_URI` respectfully

```rust
let qb = Quickbooks::new(client_id, client_secret, redirect_url, company_id, Environment::SANDBOX).await?;
```

or 

```rust
let qb = Quickbooks::new_from_env(company_id, Environment::PRODUCTION)
```

### Querying

```rust
let customers: Vec<Customer> = Customer::query(&qb, "where _ = _").await?;

// Same as above, just with the maxresults already set to 1
let customer: Customer = Customer::query_single(&qb, "where _ = _").await?;
```