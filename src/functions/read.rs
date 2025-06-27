use quickbooks_types::QBItem;
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    QBContext,
};

use super::{qb_request, QBResponse};

pub trait QBRead {
    fn read(&mut self, qb: &QBContext, client: &Agent) -> Result<(), APIError>;
}
impl<T: QBItem> QBRead for T {
    fn read(&mut self, qb: &QBContext, client: &Agent) -> Result<(), APIError> {
        qb_read(self, qb, client)
    }
}

/// Read the object by ID from quickbooks context
/// and write it to an item
fn qb_read<T: QBItem>(item: &mut T, qb: &QBContext, client: &Agent) -> Result<(), APIError> {
    let Some(id) = item.id() else {
        return Err(APIErrorInner::NoIdOnRead.into());
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None,
    )?;

    log::info!(
        "Successfully Read {} object with ID : {}",
        T::name(),
        response
            .object
            .id()
            .expect("ID should be present in the response")
    );

    *item = response.object;

    Ok(())
}

/// Retrieves an object by ID from quickbooks context
pub fn qb_get_single<T: QBItem>(id: &str, qb: &QBContext, client: &Agent) -> Result<T, APIError> {
    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None,
    )?;
    Ok(response.object)
}
