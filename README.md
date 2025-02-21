# Quick-Oxibooks: Rust Client for QuickBooks Online API

<!-- [![Crates.io](https://img.shields.io/crates/v/quick-oxibooks)](https://crates.io/crates/quick-oxibooks)
[![Documentation](https://docs.rs/quick-oxibooks/badge.svg)](https://docs.rs/quick-oxibooks)
[![License](https://img.shields.io/crates/l/quick-oxibooks)](LICENSE) -->

`quick-oxibooks` is a Rust library for interacting with the QuickBooks Online (QBO) API. It provides a high-level, type-safe, and async-first interface for managing accounting data, including invoices, customers, payments, and more. Built on top of the `quickbooks-types` crate, it simplifies integration with QuickBooks while maintaining full control over API interactions.

---

## Features

- **Full QuickBooks API Coverage**: Supports all major QuickBooks entities and operations.
- **Async-First**: Built with `async`/`await` for seamless integration into async Rust applications.
- **Type-Safe**: Strongly-typed API responses and requests to prevent runtime errors.
- **Authentication**: Handles OAuth2 authentication and token management.
- **Batch Processing**: Supports batch operations for efficient bulk data handling.
- **Error Handling**: Comprehensive error types for API, authentication, and validation errors.
- **Extensible**: Easily extendable for custom integrations and use cases.
- **Logging**: Built-in logging for debugging and monitoring API interactions.

---

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
quick-oxibooks = "0.1.0"
```

Optional features can be enabled:

```toml
[dependencies]
quick-oxibooks = { version = "0.1.0", features = ["pdf", "attachments"] }
```

### Optional Features

- **pdf**: Enables PDF generation and retrieval for supported entities (e.g., invoices, estimates).
- **attachments**: Enables support for file attachments and uploads.
- **builder**: Enables builder patterns for entity creation (requires `quickbooks-types` with the `builder` feature).

---

## Quick Start

### 1. Set Up OAuth2 Credentials

To use the QuickBooks API, you'll need:
- A QuickBooks Developer Account.
- OAuth2 `client_id`, `access_token`, and an optional `refresh_token`.
  - *warning:* this library does not handle the OAuth2 flow. You will need to implement it yourself or use a library like `oauth2` to obtain the access token and refresh token.

### 2. Create a Client

```rust
use quick_oxibooks::{QBContext, Environment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = "your_client_id";
    let client_secret = "your_client_secret";
    let redirect_uri = "your_redirect_uri";
    let company_id = "your_company_id";

    // Process these using Oauth2 flow
    // and obtain access_token and refresh_token

    let client = reqwest::Client::new();
    let qb = QBContext::new(
        Environment::Sandbox, // Use Environment::Production for live data
        company_id,
        access_token,
        refresh_token,
        &client
    ).await?;

    Ok(())
}
```

### 3. Query Data

```rust
use quick_oxibooks::{Quickbooks, Environment};
use quickbooks_types::Customer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let qb = QBContext::new_from_env(Environment::Sandbox, &client)?;

    // Query customers
    let customers: Vec<Customer> = qb.query("SELECT * FROM Customer WHERE Active = true").await?;
    println!("Found {} customers", customers.len());

    Ok(())
}
```

### 4. Create an Invoice

```rust
use quick_oxibooks::{Quickbooks, Environment};
use quickbooks_types::{Invoice, Line, LineDetail, SalesItemLineDetail, NtRef};
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let qb = QBContext::new_from_env(Environment::Sandbox, &client)?;

    let invoice = Invoice {
        customer_ref: Some(NtRef {
            value: Some("CUST123".to_string()),
            name: Some("John Doe".to_string()),
            ..Default::default()
        }),
        line: Some(vec![
            Line {
                amount: Some(100.0),
                line_detail: LineDetail::SalesItemLineDetail(SalesItemLineDetail {
                    item_ref: Some(NtRef {
                        value: Some("ITEM123".to_string()),
                        name: Some("Product A".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }
        ]),
        txn_date: Some(NaiveDate::from_ymd(2023, 10, 1)),
        ..Default::default()
    };

    let created_invoice = qb.create(&invoice).await?;
    println!("Created invoice with ID: {}", created_invoice.id.unwrap());

    Ok(())
}
```

---

## Advanced Usage

### Batch Processing

```rust
use quick_oxibooks::{Quickbooks, Environment, BatchRequest, BatchOperation, DiscoveryDoc};
use quickbooks_types::{Invoice, Customer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let qb = QBContext::new_from_env(Environment::Sandbox, &client)?;

    let mut batch = BatchRequest::new();
    batch.add_item(invoice1, BatchOperation::Create)?;
    batch.add_item(customer1, BatchOperation::Update)?;
    batch.add_query("SELECT * FROM Invoice WHERE TotalAmt > '1000.00'")?;

    let response = batch.execute(&qb, &client).await?;

    for result in response.items {
        match result.result {
            BatchResult::Success(entity) => println!("Success: {:?}", entity),
            BatchResult::Error(error) => println!("Error: {:?}", error),
            _ => {}
        }
    }

    Ok(())
}
```

### PDF Generation (Requires `pdf` Feature)

```rust
use quick_oxibooks::{Quickbooks, Environment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let qb = Quickbooks::new_from_env(Environment::Sandbox).await?;

    let invoice_id = "INV123";
    let pdf_bytes = qb.get_pdf::<Invoice>(invoice_id).await?;
    std::fs::write("invoice.pdf", pdf_bytes)?;

    println!("PDF saved to invoice.pdf");

    Ok(())
}
```

---

## Error Handling

The library provides a comprehensive `APIError` type for handling errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("API request failed: {0}")]
    RequestError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Batch processing error: {0}")]
    BatchError(String),
    // ...
}
```

---

## Contributing

We welcome contributions! Here’s how to get started:

1. **Fork the Repository**: Start by forking the [quick-oxibooks repository](https://github.com/your-repo/quick-oxibooks).
2. **Set Up the Development Environment**:
   - Clone your fork: `git clone https://github.com/your-username/quick-oxibooks.git`
   - Install dependencies: `cargo build`
3. **Make Changes**: Implement your changes or fixes.
4. **Run Tests**: Ensure all tests pass with `cargo test`.
5. **Submit a Pull Request**: Open a PR against the `main` branch with a detailed description of your changes.

### Guidelines

- Follow Rust coding conventions and best practices.
- Write unit tests for new functionality.
- Document new features and update the README if necessary.
- Use descriptive commit messages.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

<!-- ---

## Documentation

For detailed documentation, visit [docs.rs/quick-oxibooks](https://docs.rs/quick-oxibooks). -->
