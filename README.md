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

# README WIP

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
