use quickbooks_types::QBItem;
use ureq::{http::Method, Agent};

use crate::{
    error::{APIError, APIErrorInner},
    QBContext,
};

use super::{qb_request, QBResponse};

/// Trait for reading `QuickBooks` entities from the API.
///
/// This trait provides methods for reading entities from `QuickBooks`, either by
/// updating an existing entity instance or by fetching a completely new entity by ID.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement [`QBItem`].
/// You don't need to implement it manually.
///
/// # Methods
///
/// - `read()`: Updates the current entity instance with fresh data from `QuickBooks`
/// - [`qb_get_single()`]: Static function to fetch an entity by ID
///
/// # Examples
///
/// ## Reading to Update Existing Entity
///
/// ```rust
/// use quick_oxibooks::{QBContext, functions::QBRead};
/// use quickbooks_types::Customer;
/// use ureq::Agent;
///
/// let client = Agent::new_with_defaults();
/// let qb_context = QBContext::new(/* ... */)?;
///
/// // Entity with ID that needs fresh data
/// let mut customer = Customer::default();
/// customer.id = Some("123".to_string());
///
/// // Read fresh data from QuickBooks into this entity
/// customer.read(&qb_context, &client)?;
/// println!("Updated customer: {}", customer.display_name.unwrap_or_default());
/// ```
///
/// ## Fetching New Entity by ID
///
/// ```rust
/// use quick_oxibooks::functions::read::qb_get_single;
/// use quickbooks_types::Customer;
///
/// // Fetch a customer by ID
/// let customer: Customer = qb_get_single("123", &qb_context, &client)?;
/// println!("Fetched customer: {}", customer.display_name.unwrap_or_default());
/// ```
///
/// # Errors
///
/// - `NoIdOnRead`: Entity doesn't have an ID for reading
/// - `UreqError`: Network or HTTP errors during API call  
/// - `BadRequest`: `QuickBooks` API returned an error (e.g., entity not found)
/// - `JsonError`: Response parsing errors
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
        None::<std::iter::Empty<(&str, &str)>>,
    )?;

    #[cfg(feature = "logging")]
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

/// Retrieves a `QuickBooks` entity by ID.
///
/// This function fetches a single entity from `QuickBooks` using its unique identifier.
/// Unlike the `read` method on [`QBRead`], this is a standalone function that creates
/// a new entity instance rather than updating an existing one.
///
/// # Parameters
///
/// - `id`: The `QuickBooks` entity ID to fetch
/// - `qb`: `QuickBooks` context with authentication and configuration
/// - `client`: HTTP client for making the API request
///
/// # Returns
///
/// Returns the fetched entity with all fields populated from `QuickBooks`.
///
/// # Type Parameter
///
/// `T` must implement [`QBItem`] - this determines which type of entity to fetch
/// and which API endpoint to use.
///
/// # Examples
///
/// ```rust
/// use quick_oxibooks::functions::read::qb_get_single;
/// use quickbooks_types::{Customer, Invoice, Item};
///
/// // Fetch different types of entities
/// let customer: Customer = qb_get_single("123", &qb_context, &client)?;
/// let invoice: Invoice = qb_get_single("456", &qb_context, &client)?;
/// let item: Item = qb_get_single("789", &qb_context, &client)?;
/// ```
///
/// # Errors
///
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: `QuickBooks` API returned an error (e.g., entity not found, invalid ID)
/// - `JsonError`: Response parsing errors
/// - Rate limiting errors if API limits are exceeded
pub fn qb_get_single<T: QBItem>(id: &str, qb: &QBContext, client: &Agent) -> Result<T, APIError> {
    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<&()>,
        None,
        None::<std::iter::Empty<(&str, &str)>>,
    )?;
    Ok(response.object)
}
