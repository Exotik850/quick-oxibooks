use quickbooks_types::{QBCreatable, QBItem};
use reqwest::{Client, Method};

use crate::{
    error::APIError,
    functions::{qb_request, QBResponse},
    QBContext,
};

pub trait QBCreate {
    fn create(
        &self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<Self, APIError>> 
    where
        Self: Sized; 
}
impl<T: QBItem + QBCreatable> QBCreate for T {
    async fn create(&self, qb: &QBContext, client: &Client) -> Result<Self, APIError> {
        qb_create(self, qb, client).await
    } 
}

/// Creates the given item using the context given, but first
/// checks if the item is suitable to be created.
async fn qb_create<T: QBItem + QBCreatable>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    if !item.can_create() {
        return Err(APIError::CreateMissingItems);
    }

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}", qb.company_id, T::qb_id()),
        Some(item),
        None,
        None,
    )
    .await?;

    log::info!(
        "Successfully created {} with ID of '{:?}'",
        T::name(),
        response.object.id().into_iter().next()
    );

    Ok(response.object)
}
