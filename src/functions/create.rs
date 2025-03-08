use quickbooks_types::{QBCreatable, QBItem};
use reqwest::{Client, Method};

use crate::{
    error::APIError,
    functions::{qb_request, QBResponse},
    QBContext,
};

/// Creates the given item using the context given, but first
/// checks if the item is suitable to be created.
pub async fn qb_create<T: QBItem + QBCreatable>(
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
