use quickbooks_types::{QBDeletable, QBItem};
use serde::{Deserialize, Serialize};
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    functions::{qb_request, QBResponse},
    APIResult, QBContext,
};

/// Trait for deleting `QuickBooks` entities via the API.
///
/// This trait provides the `delete` method for removing entities from `QuickBooks`.
/// It validates that entities have the required ID and sync token before attempting deletion.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement both
/// [`QBItem`] and [`QBDeletable`]. You don't need to implement it manually.
///
/// # Requirements
///
/// Before deletion, entities must have:
/// - Valid ID (entity exists in `QuickBooks`)
/// - Current sync token (for optimistic concurrency control)
///
/// These are automatically present when entities are read from `QuickBooks`.
///
/// # Important Notes
///
/// - **Permanent Action**: Deletion cannot be undone
/// - **Referential Integrity**: `QuickBooks` may prevent deletion if entity is referenced elsewhere
/// - **Audit Trail**: Some entities may be marked as inactive instead of deleted
/// - **Sync Token**: Must be current or deletion will fail with sync error
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::delete::QBDelete;
/// use quick_oxibooks::functions::query::QBQuery;
/// use quickbooks_types::Invoice;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
///
/// // Find and delete an invoice
/// let invoice = Invoice::query_single(
///     "WHERE DocNumber = 'INV-0001'",
///     &qb_context,
///     &client
/// ).unwrap();
///
/// // Delete the invoice
/// let deleted_info = invoice.delete(&qb_context, &client).unwrap();
/// println!("Deleted invoice with ID: {}", deleted_info.id);
/// ```
///
/// # Return Value
///
/// Returns [`QBDeleted`] with information about the deleted entity:
/// - `id`: The ID of the deleted entity
/// - `status`: Deletion status from `QuickBooks`
/// - `domain`: The `QuickBooks` domain information
///
/// # Errors
///
/// - `DeleteMissingItems`: Entity missing ID or sync token
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: `QuickBooks` API error (e.g., entity referenced elsewhere, sync conflict)
/// - `JsonError`: Response parsing errors
pub trait QBDelete {
    /// Deletes the item
    /// returns an error if the item has no ID and sync token
    /// available or if the request itself fails
    fn delete(&self, qb: &QBContext, client: &Agent) -> APIResult<QBDeleted>
    where
        Self: Sized;
}

impl<T: QBItem + QBDeletable> QBDelete for T {
    fn delete(&self, qb: &QBContext, client: &Agent) -> APIResult<QBDeleted> {
        qb_delete(self, qb, client)
    }
}

/// Deletes the given item using the ID
/// returns an error if the item has no ID and sync token
/// available or if the request itself fails
fn qb_delete<T: QBItem + QBDeletable>(
    item: &T,
    qb: &QBContext,
    client: &Agent,
) -> Result<QBDeleted, APIError> {
    let (Some(_), Some(_)) = (item.sync_token(), item.id()) else {
        return Err(APIErrorInner::DeleteMissingItems.into());
    };

    let delete_object: QBToDelete = item.to_delete();

    let response: QBResponse<QBDeleted> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}?operation=delete", qb.company_id, T::qb_id()),
        Some(&delete_object),
        None,
        None::<std::iter::Empty<(&str, &str)>>,
    )?;

    #[cfg(feature = "logging")]
    log::info!("Successfully deleted {} with ID of {}", T::name(), id);

    Ok(response.object)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct QBToDelete<'a> {
    id: &'a str,
    sync_token: &'a str,
}

trait QBToDeleteTrait {
    fn id(&self) -> &str;
    fn sync_token(&self) -> &str;
    fn to_delete(&self) -> QBToDelete<'_> {
        QBToDelete {
            id: self.id(),
            sync_token: self.sync_token(),
        }
    }
}
impl<T: QBItem> QBToDeleteTrait for T {
    fn id(&self) -> &str {
        self.id().expect("Tried to delete an object with no ID")
    }

    fn sync_token(&self) -> &str {
        self.sync_token()
            .expect("Tried to delete an object with no SyncToken")
    }
}

/// Information about a successfully deleted `QuickBooks` entity.
///
/// This struct contains metadata returned by `QuickBooks` after a successful deletion operation.
/// It provides confirmation details about what was deleted and the operation status.
///
/// # Fields
///
/// - `status`: The status of the deletion operation (typically "Deleted")
/// - `domain`: `QuickBooks` domain information (e.g., "QBO" for `QuickBooks` Online)
/// - `id`: The ID of the entity that was deleted
///
/// # Examples
///
/// ```no_run
/// use quick_oxibooks::{QBContext, Environment};
/// use quick_oxibooks::functions::delete::{QBDelete, QBDeleted};
/// use quick_oxibooks::functions::query::QBQuery;
/// use quickbooks_types::Invoice;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(
///     Environment::SANDBOX,
///     "company_id".to_string(),
///     "access_token".to_string(),
///     &client,
/// ).unwrap();
/// // Assume `entity` is fetched and has id/sync token
/// let entity = Invoice::query_single("WHERE DocNumber = 'INV-0001'", &qb_context, &client).unwrap();
/// let deleted_info: QBDeleted = entity.delete(&qb_context, &client).unwrap();
///
/// println!("Deletion status: {}", deleted_info.status);
/// println!("Deleted entity ID: {}", deleted_info.id);
/// println!("Domain: {}", deleted_info.domain);
/// ```
///
/// # Usage
///
/// This struct is returned by the [`QBDelete::delete`] method and can be used to:
/// - Confirm successful deletion
/// - Log deletion operations for audit purposes
/// - Verify the correct entity was deleted by checking the ID
#[derive(Deserialize, Debug, Default)]
pub struct QBDeleted {
    pub status: String,
    pub domain: String,
    #[serde(rename = "Id")]
    pub id: String,
}
