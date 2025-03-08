use quickbooks_types::QBItem;
use reqwest::{Client, Method};

use crate::{error::APIError, QBContext};

use super::{qb_request, QBResponse};

/// Read the object by ID from quickbooks context
/// and write it to an item
pub async fn qb_read<T: QBItem>(
    item: &mut T,
    qb: &QBContext,
    client: &Client,
) -> Result<(), APIError> {
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnRead);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None,
    )
    .await?;

    log::info!(
        "Successfully Read {} object with ID : {}",
        T::name(),
        response
            .object
            .id()
            .unwrap_or(&"No ID after reading QB Object".into())
    );

    *item = response.object;

    Ok(())
}

/// Retrieves an object by ID from quickbooks context
pub async fn qb_get_single<T: QBItem>(
    id: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None,
    )
    .await?;
    Ok(response.object)
}
