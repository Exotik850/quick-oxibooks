use quickbooks_types::{QBCreatable, QBItem};
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    functions::{qb_request, QBResponse},
    APIResult, QBContext,
};

/// Trait for creating an item
pub trait QBCreate {
    /// Creates the item
    /// returns an error if the item is not suitable for creation
    /// or if the request itself fails
    fn create(&self, qb: &QBContext, client: &Agent) -> APIResult<Self>
    where
        Self: Sized;
}
impl<T: QBItem + QBCreatable> QBCreate for T {
    fn create(&self, qb: &QBContext, client: &Agent) -> Result<Self, APIError> {
        qb_create(self, qb, client)
    }
}

/// Creates the given item using the context given, but first
/// checks if the item is suitable to be created.
fn qb_create<T: QBItem + QBCreatable>(
    item: &T,
    qb: &QBContext,
    client: &Agent,
) -> Result<T, APIError> {
    if !item.can_create() {
        return Err(APIErrorInner::CreateMissingItems.into());
    }

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}", qb.company_id, T::qb_id()),
        Some(item),
        None,
        None::<std::iter::Empty<(&str, &str)>>,
    )?;

    log::info!(
        "Successfully created {} with ID of '{:?}'",
        T::name(),
        response.object.id().into_iter().next()
    );

    Ok(response.object)
}
