use quickbooks_types::{QBCreatable, QBItem};
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    functions::{qb_request, QBResponse},
    APIResult, QBContext,
};

/// Trait for creating QuickBooks entities via the API.
///
/// This trait provides the `create` method for sending new entities to QuickBooks.
/// It automatically validates that entities meet creation requirements before
/// sending them to the API.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement both
/// [`QBItem`] and [`QBCreatable`]. You don't need to implement it manually.
///
/// # Validation
///
/// Before creating, the trait checks `can_create()` to ensure the entity has
/// all required fields. If validation fails, returns `CreateMissingItems` error.
///
/// # Examples
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::QBCreate};
/// use quickbooks_types::Customer;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Create a new customer
/// let mut customer = Customer::default();
/// customer.display_name = Some("John Doe".to_string());
/// customer.email = Some("john@example.com".to_string());
///
/// // Send to QuickBooks
/// let created_customer = customer.create(&qb_context, &client)?;
/// println!("Created customer with ID: {:?}", created_customer.id());
/// ```
///
/// # Return Value
///
/// Returns the created entity with QuickBooks-assigned ID, sync token, and metadata.
/// This return value can be used for subsequent operations like updates or deletes.
///
/// # Errors
///
/// - `CreateMissingItems`: Entity doesn't meet creation requirements
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: QuickBooks API returned an error response
/// - `JsonError`: Response parsing errors
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
