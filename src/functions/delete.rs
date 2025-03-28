use http_client::{http_types::Method, HttpClient};
use quickbooks_types::{QBDeletable, QBItem};
use serde::{Deserialize, Serialize};

use crate::{
    error::APIError, functions::{qb_request, QBResponse}, QBContext
};

/// Trait for deleting an item
pub trait QBDelete {
    /// Deletes the item
    /// returns an error if the item has no ID and sync token
    /// available or if the request itself fails
    fn delete<Client: HttpClient>(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<QBDeleted, APIError>>
    where
        Self: Sized;
}

impl<T: QBItem + QBDeletable> QBDelete for T {
    fn delete<Client: HttpClient>(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<QBDeleted, APIError>> {
        qb_delete(self, qb, client)
    }
}

/// Deletes the given item using the ID
/// returns an error if the item has no ID and sync token
/// available or if the request itself fails
async fn qb_delete<T, Client>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<QBDeleted, APIError>
where
    T: QBItem + QBDeletable,
    Client: HttpClient,
{
    let (Some(_), Some(id)) = (item.sync_token(), item.id()) else {
        return Err(APIError::DeleteMissingItems);
    };

    let delete_object: QBToDelete = item.to_delete();

    let response: QBResponse<QBDeleted> = qb_request(
        qb,
        client,
        Method::Post,
        &format!("company/{}/{}?operation=delete", qb.company_id, T::qb_id()),
        Some(&delete_object),
        None,
        None,
    )
    .await?;

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
