use quickbooks_xsd::intuit_base_types::{
    AttachableRefType as QBAttachableRef, ReferenceTypeType as QBRef,
};

/// Error returned when an entity cannot be converted to a reference because it
/// has no ID set.
#[derive(Debug)]
pub struct NoIdForRef;

impl std::fmt::Display for NoIdForRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entity has no ID, cannot build a reference")
    }
}

impl std::error::Error for NoIdForRef {}

pub trait QBToRef {
    /// Converts the entity into a `ReferenceType` pointing at it.
    ///
    /// # Errors
    ///
    /// Returns [`NoIdForRef`] if the entity has no ID set.
    fn to_ref(&self) -> Result<QBRef, NoIdForRef>;
}

/// Trait for entities that can be converted into an [`QBAttachableRef`] for linking
/// an attachment to that entity.
///
/// This is automatically implemented for every type that implements [`QBToRef`].
/// Use [`QBToAttachableRef::to_attachable_ref`] when building the `attachable_ref`
/// list of an `Attachable` prior to upload.
pub trait QBToAttachableRef: QBToRef {
    /// Converts the entity into an `AttachableRef` pointing at it.
    ///
    /// # Errors
    ///
    /// Returns [`NoIdForRef`] if the entity has no ID set.
    fn to_attachable_ref(&self) -> Result<QBAttachableRef, NoIdForRef> {
        let entity_ref = self.to_ref()?;
        Ok(QBAttachableRef {
            entity_ref: Some(entity_ref),
            line_info: None,
            include_on_send: None,
            custom_field: Vec::new(),
            attachable_ref_ex: None,
        })
    }
}

impl<T: QBToRef> QBToAttachableRef for T {}

macro_rules! impl_qb_to_ref {
  ($($struct:ident {$name_field:ident}),+) => {
    $(
      impl QBToRef for $struct {
        fn to_ref(&self) -> Result<QBRef, NoIdForRef> {
          if let Some(id) = self.id() {
            Ok(QBRef {
              type_: Some(Self::NAME.into()),
              name: self.$name_field.clone(),
              content: id.to_string(),
            })
          } else {
            Err(NoIdForRef)
          }
        }
      }
    )+
  }
}

macro_rules! impl_qb_to_ref_func {
    ($($struct:ident { $name_field:ident }),+) => {
        $(
            impl QBToRef for $struct {
                fn to_ref(&self) -> Result<QBRef, NoIdForRef> {
                    if let Some(id) = self.id() {
                        Ok(QBRef {
                            type_: Some(Self::NAME.into()),
                            name: self.$name_field().cloned(),
                            content: id.to_string(),
                        })
                    } else {
                        Err(NoIdForRef)
                    }
                }
            }
        )+
    };
}

use crate::traits::QBItem;
use quickbooks_xsd::intuit_rest_service_def::{
    Account, Attachable, Customer, Invoice, Item, SalesReceipt, Vendor,
};

//TODO: Add more types as needed
impl_qb_to_ref_func!(
    Invoice { doc_number },
    SalesReceipt { doc_number },
    Customer { display_name }
);
impl_qb_to_ref!(
    Account {
        fully_qualified_name
    },
    Attachable { file_name },
    Item { name },
    Vendor { display_name }
);
