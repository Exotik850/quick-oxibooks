use quickbooks_types::{QBDeletable, QBItem};
use serde::{Deserialize, Serialize};
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    functions::{qb_request, QBResponse},
    APIResult, QBContext,
};

/// Trait for deleting an item
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
    let (Some(_), Some(id)) = (item.sync_token(), item.id()) else {
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
        None,
    )?;

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
    fn to_delete(&self) -> QBToDelete {
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

/// Information about the deleted object from `qb_delete`
#[derive(Deserialize, Debug, Default)]
pub struct QBDeleted {
    pub status: String,
    pub domain: String,
    #[serde(rename = "Id")]
    pub id: String,
}
