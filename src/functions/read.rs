use http_client::{http_types::Method, HttpClient};
use quickbooks_types::QBItem;

use crate::{error::APIError, QBContext};

use super::{qb_request, QBResponse};

pub trait QBRead {
    fn read<Client: HttpClient>(
        &mut self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<(), APIError>>;
}
impl<T: QBItem> QBRead for T {
    fn read<Client: HttpClient>(
        &mut self,
        qb: &QBContext,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<(), APIError>> {
        qb_read(self, qb, client)
    }
}

/// Read the object by ID from quickbooks context
/// and write it to an item
async fn qb_read<T, Client>(item: &mut T, qb: &QBContext, client: &Client) -> Result<(), APIError>
where
    T: QBItem,
    Client: HttpClient,
{
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnRead);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::Get,
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
pub async fn qb_get_single<T, Client>(
    id: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError>
where
    T: QBItem,
    Client: HttpClient,
{
    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::Get,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None,
    )
    .await?;
    Ok(response.object)
}
