# Quick-Oxibooks: Rust Client for QuickBooks Online (QBO)

The `quick-oxibooks` crate is a small, ergonomic client for the QuickBooks Online API built on top of `quickbooks-types`. It focuses on a simple, type-safe workflow for CRUD, queries, reports, and batch operations, with rate-limiting and OAuth2 token usage handled by a lightweight context.

- Crate: quick-oxibooks
- Version: 0.1.1
- License: MIT

QuickBooks API docs: https://developer.intuit.com/app/developer/qbo/docs/get-started

---

## Status and scope

This crate:

- Provides a minimal blocking HTTP client for QBO built on `ureq`
- Exposes a `QBContext` and `Environment` for auth/config + rate limiting
- Implements high-level traits for:
  - CRUD: create, read, delete
  - Query: SQL-like QBO queries
  - Reports: typed report fetching using `quickbooks-types::reports`
  - Batch: batched create/update/delete/query
- Re-exports QBO data models via `quick_oxibooks::types::*` (from `quickbooks-types`)
- Offers optional features for attachments, PDFs, logging, macros, and Polars (via `quickbooks-types`)

This crate does not:

- Implement the OAuth browser flow (you supply tokens)
- Provide an async runtime (it’s synchronous via `ureq`)
- Hide QBO’s SQL-like query strings behind a DSL

---

## Installation

```/dev/null/Cargo.toml#L1-8
[dependencies]
quick-oxibooks = "0.1.0"
```

Optional features:

- attachments: enable file upload/download helpers
- pdf: enable PDF retrieval for supported entities
- macros: enable query-building convenience macros
- logging: enable request/response logging via the `log` crate
- polars: pass-through feature that enables Polars helpers in `quickbooks-types`

```/dev/null/Cargo.toml#L10-15
[dependencies]
quick-oxibooks = { version = "0.1.0", features = ["attachments", "pdf", "logging"] }
# or
quick-oxibooks = { version = "0.1.0", features = ["polars"] }
```

---

## Modules and re-exports

- `quick_oxibooks::client`:
  - `QBContext`, `RefreshableQBContext`, `Environment`
- `quick_oxibooks::functions`:
  - `create::QBCreate`, `read::QBRead`, `delete::QBDelete`
  - `query::QBQuery`
  - `reports::QBReport`
  - `attachment` (feature = "attachments"), `pdf` (feature = "pdf")
- `quick_oxibooks::batch`:
  - `QBBatchOperation`, `BatchIterator`
- `quick_oxibooks::error`:
  - `APIError`, `APIErrorInner`
- `quick_oxibooks::types::*`:
  - All re-exports from `quickbooks-types` (entities, reports, helpers)

---

## Quick start

Create a context, then perform CRUD and queries with typed entities from `quickbooks-types`.

```/dev/null/quickstart.rs#L1-60
use quick_oxibooks::{Environment, QBContext};
use quick_oxibooks::functions::{create::QBCreate, read::QBRead, delete::QBDelete, query::QBQuery};
use quick_oxibooks::error::APIError;
use quick_oxibooks::types::{Customer, Invoice};
use ureq::Agent;

fn main() -> Result<(), APIError> {
    // 1) Create a QBO context (provide your existing OAuth2 Bearer token and company ID)
    let client = Agent::new_with_defaults();
    let qb = QBContext::new(
        Environment::SANDBOX,
        "your_company_id".to_string(),
        "your_access_token".to_string(),
        &client,
    )?;

    // 2) Create
    let mut customer = Customer::default();
    customer.display_name = Some("Acme Corp".into());
    let created = customer.create(&qb, &client)?;
    println!("Created customer ID = {:?}", created.id);

    // 3) Read (in-place refresh by ID)
    let mut c = Customer::default();
    c.id = created.id.clone();
    c.read(&qb, &client)?;
    println!("Refreshed name = {:?}", c.display_name);

    // 4) Query (SQL-like)
    let invoices: Vec<Invoice> = Invoice::query(
        "WHERE TotalAmt > '100.00' ORDER BY MetaData.CreateTime DESC",
        Some(10),
        &qb,
        &client,
    )?;
    println!("Found {} invoices over $100", invoices.len());

    // 5) Delete (requires ID + sync_token)
    // let deleted = created.delete(&qb, &client)?;
    // println!("Deleted status = {}", deleted.status);

    Ok(())
}
```

Tip: You can also build contexts from env vars: `QBContext::new_from_env(Environment::..., &client)` expects `QB_COMPANY_ID` and `QB_ACCESS_TOKEN`.

---

## Reports

Use strongly-typed report kinds and optional typed parameters from `quickbooks-types::reports`.

```/dev/null/reports.rs#L1-50
use chrono::NaiveDate;
use quick_oxibooks::{Environment, QBContext};
use quick_oxibooks::functions::reports::QBReport;
use quick_oxibooks::types::reports::{Report, types::*, params::*};
use ureq::Agent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Agent::new_with_defaults();
    let qb = QBContext::new(
        Environment::SANDBOX,
        "company".into(),
        "access_token".into(),
        &client,
    )?;

    // Example: Balance Sheet with typed params
    let params = BalanceSheetParams::new()
        .start_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        .accounting_method(AccountingMethod::Accrual)
        .summarize_column_by(SummarizeColumnBy::Month);

    let report: Report = Report::get(&qb, &client, &BalanceSheet, Some(params))?;
    println!("Report name = {:?}", report.name());

    Ok(())
}
```

---

## Batch

Batch multiple operations (query/create/update/delete) into one request and correlate responses.

```/dev/null/batch.rs#L1-90
use quick_oxibooks::{QBContext, Environment};
use quick_oxibooks::batch::{QBBatchOperation, BatchIterator};
use quick_oxibooks::types::{Invoice, Vendor};
use ureq::Agent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Agent::new_with_defaults();
    let qb = QBContext::new(
        Environment::SANDBOX,
        "company".into(),
        "token".into(),
        &client,
    )?;

    // Prepare a few operations
    let ops = vec![
        // Query invoices
        QBBatchOperation::query("SELECT * FROM Invoice WHERE TotalAmt > '250.00' MAXRESULTS 5"),
        // Create a vendor
        QBBatchOperation::create(Vendor {
            display_name: Some("New Supplier LLC".into()),
            ..Default::default()
        }),
        // Update an invoice (provide id + sync_token as needed)
        QBBatchOperation::update(Invoice {
            id: Some("123".into()),
            // .. other fields to change here
            ..Default::default()
        }),
    ];

    // Execute and inspect
    let results = ops.batch(&qb, &client)?;
    for (op, resp) in results {
        println!("{op:?} -> {resp:?}");
    }

    Ok(())
}
```

---

## Features

- attachments: upload and manage file attachments
- pdf: fetch PDFs for supported types (e.g., invoices)
- logging: enable request/response logs via `log`
- macros: convenience macros for building queries
- polars: enable `quickbooks-types` Polars helpers (feature passthrough)

Enable one or more:

```/dev/null/Cargo.toml#L1-6
[dependencies]
quick-oxibooks = { version = "0.1.0", features = ["attachments", "pdf", "logging"] }
```

---

## Errors

`APIError` wraps all errors surfaced by the client, including HTTP, JSON, and QBO faults, with variants in `APIErrorInner` such as:

- UreqError / HttpError / JsonError
- BadRequest(QBErrorResponse) with QBO fault info
- CreateMissingItems, DeleteMissingItems, NoIdOnRead/Send/GetPDF
- ThrottleLimitReached, BatchLimitExceeded
- EnvVarError, InvalidClient, etc.

```/dev/null/errors.rs#L1-40
use quick_oxibooks::error::{APIError, APIErrorInner};

fn handle(err: APIError) {
    match &*err {
        APIErrorInner::BadRequest(qb) => {
            eprintln!("QBO error: {}", qb);
        }
        APIErrorInner::ThrottleLimitReached => {
            eprintln!("Hit QBO rate limits; wait ~60s before retrying");
        }
        other => {
            eprintln!("Other error: {other}");
        }
    }
}
```

---

## Tips

- Auth: You supply the OAuth2 access token; `RefreshableQBContext` can renew it if you have a refresh token.
- Rate limits:
  - Regular API: 500 requests/min
  - Batch: 40 batches/min (30 ops/batch)
- Field names in queries use QBO PascalCase (e.g., `DisplayName`, `TotalAmt`).
- IDs + `sync_token` are required for deletion and some updates.
- This client is blocking (ureq). For async, consider wrapping calls in a thread pool.

---

## Contributing

Issues and PRs are welcome. If you need additional endpoints, features, or ergonomics, open an issue describing your use case.

Basic workflow:
1) Fork and clone
2) `cargo build && cargo test`
3) Make changes with tests
4) Open a PR against main with context and examples

---

## Documentation

- API docs: https://docs.rs/quick-oxibooks
- Types crate: https://docs.rs/quickbooks-types

---

## License

MIT. See LICENSE.md for details.
