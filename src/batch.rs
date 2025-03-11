//! # `QuickBooks` Online Batch Request Module
//!
//! This module provides functionality for making batch requests to the `QuickBooks` Online API.
//! Batch requests allow you to combine multiple operations into a single API call,
//! which can help improve performance and reduce rate limiting issues.
//!
//! ## Key Features
//!
//! - Execute multiple `QuickBooks` operations in a single API call
//! - Support for create, update, delete operations on various resource types
//! - Support for query operations to fetch multiple resources at once
//! - Type-safe API with enum-based resource handling
//!
//! ## Usage Example
//!
//! ```rust
//! use quick_oxibooks::{
//!     batch::{QBBatchOperation, BatchIterator},
//!     QBContext,
//! };
//! use quickbooks_types::{Invoice, Vendor};
//!
//! async fn batch_example(qb: &QBContext, client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a collection of operations
//!     let operations = vec![
//!         // Query for invoices
//!         QBBatchOperation::query("SELECT * FROM Invoice WHERE TotalAmt > '100.00' MAXRESULTS 10"),
//!         
//!         // Create a new vendor
//!         QBBatchOperation::create(Vendor {
//!             display_name: Some("New Supplier Inc.".to_string()),
//!             ..Default::default()
//!         }),
//!         
//!         // Update an existing invoice
//!         QBBatchOperation::update(Invoice {
//!             id: Some("123".to_string()),
//!             // ... other fields
//!             ..Default::default()
//!         }),
//!     ];
//!
//!     // Execute the batch request
//!     let results = operations.batch(qb, client).await?;
//!     
//!     // Process the results
//!     for (operation, response) in results {
//!         // Handle each response based on the operation type
//!         match response {
//!             // Handle different response types...
//!             _ => println!("Got a response!"),
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Best Practices
//!
//! - Combine related operations in a single batch to minimize API calls
//! - Keep batch sizes reasonable (`QuickBooks` allows no more than 30 operations per batch, 40 req / min)
//! - Handle potential partial failures where some operations succeed while others fail
//! - Use the appropriate operation type (create, update, delete, query) for each task
//!
use std::{collections::HashMap, future::Future};

use quickbooks_types::{Invoice, SalesReceipt, Vendor};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    error::{APIError, BatchMissingItemsError, Fault},
    functions::execute_request,
    QBContext,
};

/// Batch request structure for `QuickBooks` Online API
///
/// Internal use only, not meant to be used directly.
#[derive(Serialize, Deserialize, Debug)]
struct QBBatchRequest {
    #[serde(rename = "BatchItemRequest")]
    items: Vec<QBBatchItem<QBBatchOperation>>,
}

/// Represents a resource operation in a batch request
#[derive(Serialize, Deserialize, Debug)]
pub struct QBResourceOperation {
    #[serde(flatten)]
    pub resource: QBResource,
    pub operation: QBOperationType,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum QBOperationType {
    Create,
    Update,
    Delete,
}

/// Represents a batch item in a batch request,
/// Essentially adds a unique identifier to each item.
#[derive(Serialize, Deserialize, Debug)]
pub struct QBBatchItem<T> {
    #[serde(rename = "bId")]
    pub b_id: String,
    #[serde(flatten)]
    pub item: T,
}

/// Represents a batch operation, which can be either a query or a resource operation.
#[derive(Serialize, Deserialize, Debug)]
pub enum QBBatchOperation {
    Query(String),
    #[serde(untagged)]
    Operation(QBResourceOperation),
}

impl QBBatchOperation {
    pub fn query(query: impl std::fmt::Display) -> Self {
        QBBatchOperation::Query(query.to_string())
    }

    #[must_use]
    pub fn create(resource: impl Into<QBResource>) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource: resource.into(),
            operation: QBOperationType::Create,
        })
    }

    #[must_use]
    pub fn update(resource: impl Into<QBResource>) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource: resource.into(),
            operation: QBOperationType::Update,
        })
    }

    #[must_use]
    pub fn delete(resource: impl Into<QBResource>) -> Self {
        QBBatchOperation::Operation(QBResourceOperation {
            resource: resource.into(),
            operation: QBOperationType::Delete,
        })
    }
}

/// Represents a resource in a batch request,
/// TODO, Make this more generic as needed.
#[derive(Serialize, Deserialize, Debug)]
pub enum QBResource {
    SalesReceipt(SalesReceipt),
    Invoice(Invoice),
    Vendor(Vendor),
    // TODO Add more as needed
}

/// Represents the result of a query operation in a batch request.
/// TODO, Make this more generic as needed.
#[derive(Serialize, Deserialize, Debug)]
pub enum QBQueryResource {
    SalesReceipt(Vec<SalesReceipt>),
    Invoice(Vec<Invoice>),
    // TODO Add more as needed
}

/// Represents the result of a query operation in a batch request.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QBQueryResult {
    pub start_position: Option<usize>,
    pub max_results: Option<usize>,
    pub total_count: Option<usize>,
    // resource: Vec<T>,
    #[serde(flatten)]
    pub data: Option<QBQueryResource>,
}

/// Represents the response data for a batch request item.
#[derive(Serialize, Deserialize, Debug)]
pub enum QBBatchResponseData {
    Item(QBResource),
    Fault(Fault),
    QueryResponse(QBQueryResult),
}

/// Represents the response structure for a batch request.
///
/// Internal use only, not meant to be used directly.
#[derive(Serialize, Deserialize, Debug)]
struct BatchResponseExt {
    time: String,
    #[serde(rename = "BatchItemResponse")]
    items: Vec<QBBatchItem<QBBatchResponseData>>,
}

/// `BatchIterator` trait for iterating over batch operations.
///
/// Allows for executing batch requests in a more ergonomic way.
pub trait BatchIterator {
    /// Executes a batch request, returning a future that resolves to the result.
    ///
    /// The result is a vector of tuples, where each tuple consists of a `QBBatchOperation` and its corresponding `QBBatchResponseData`.
    ///
    /// If the request fails or some items are missing in the response, an `APIError` is returned.
    fn batch(
        self,
        qb: &QBContext,
        client: &reqwest::Client,
    ) -> impl Future<Output = Result<Vec<(QBBatchOperation, QBBatchResponseData)>, APIError>>;
}

impl<I> BatchIterator for I
where
    I: IntoIterator<Item = QBBatchOperation>,
{
    fn batch(
        self,
        qb: &QBContext,
        client: &reqwest::Client,
    ) -> impl Future<Output = Result<Vec<(QBBatchOperation, QBBatchResponseData)>, APIError>> {
        qb_batch(self, qb, client)
    }
}

/// Executes a batch request to `QuickBooks` Online API.
///
/// # Parameters
/// - `items`: An iterator of `QBBatchOperation` items to be included in the batch request.
/// - `client`: A reference to the `reqwest::Client` for making HTTP requests.
///
/// # Returns
/// A `Result` containing a vector of tuples, where each tuple consists of a `QBBatchOperation` and its corresponding `QBBatchResponseData`.
/// If the request fails or some items are missing in the response, an `APIError` is returned.
pub async fn qb_batch<I>(
    items: I,
    qb: &QBContext,
    client: &reqwest::Client,
    // ) -> Result<Vec<QBBatchItem<QBBatchResponseData>>, APIError>
) -> Result<Vec<(QBBatchOperation, QBBatchResponseData)>, APIError>
where
    I: IntoIterator<Item = QBBatchOperation>,
{
    let batch = QBBatchRequest {
        items: items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let b_id = format!("bId{}", i + 1);
                QBBatchItem { b_id, item }
            })
            .collect(),
    };
    let url = format!("company/{}/batch", qb.company_id);
    let resp = qb
        .with_batch_permission(|qb| {
            execute_request(qb, client, Method::POST, &url, Some(&batch), None, None)
        })
        .await?;
    let batch_resp: BatchResponseExt = resp.json().await?;
    let mut items = batch
        .items
        .into_iter()
        .map(|item| (item.b_id, item.item))
        .collect::<HashMap<_, _>>();
    let mut results = Vec::new();
    for resp_item in batch_resp.items {
        if let Some(req_item) = items.remove(&resp_item.b_id) {
            results.push((req_item, resp_item.item));
        }
    }

    if !items.is_empty() {
        return Err(APIError::BatchRequestMissingItems(BatchMissingItemsError {
            items,
            results,
        }));
    }

    Ok(results)
}

#[cfg(test)]
mod test {
    use super::{BatchResponseExt, QBBatchRequest};
    #[test]
    fn test_batch_resp() {
        let s = include_str!("../test/data/batch_resp.json");
        let resp: BatchResponseExt = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }

    #[test]
    fn test_batch_req() {
        let s = include_str!("../test/data/batch_req.json");
        let resp: QBBatchRequest = serde_json::from_str(s).unwrap();
        println!("{resp:#?}");
    }
}
