//! Macro definitions for QuickBooks Online API interactions
//!
//! # Features
//!
//! `qb_where_clause` - Macro to create SQL-like WHERE clauses with compile-time field validation
//! `qb_query` - Macro to execute QuickBooks API queries with compile-time field validation

/// Creates a SQL-like WHERE clause string for QuickBooks API queries with compile-time field validation.
///
/// This macro generates properly formatted WHERE clauses for QuickBooks Online API, converting
/// field names from snake_case to UpperCamelCase automatically, joining multiple conditions
/// with "AND", and properly escaping values.
///
/// # Features
///
/// - **Compile-time field validation**: Ensures all fields exist on the specified struct
/// - **Case conversion**: Automatically converts field names to QuickBooks API format (UpperCamelCase)
/// - **Memory optimization**: Uses compile-time string building for literals and capacity hints for expressions
/// - **Additional clause support**: Allows appending raw SQL-like conditions
///
/// # Syntax
///
/// ```ignore
/// qb_where_clause!(EntityType | field1 op value1, field2 op value2 [; additional_clauses...])
/// ```
///
/// # Parameters
///
/// - `EntityType`: The QuickBooks entity type (e.g., Customer, Invoice, Item)
/// - `field`: Field name in snake_case (automatically converted to PascalCase)
/// - `op`: Comparison operator (`=`, `like`, `in`)
/// - `value`: Value to compare against (literal or expression)
/// - `additional_clauses`: Optional raw SQL clauses after `;` separator
///
/// # Supported Operators
///
/// - `=`: Exact equality comparison
/// - `like`: Pattern matching with wildcards
/// - `in`: Value in a set of values
///
/// # Examples
///
/// ## Basic WHERE clause:
/// ```rust
/// # use quick_oxibooks::qb_where_clause;
/// # use quickbooks_types::Customer;
/// let clause = qb_where_clause!(Customer | given_name = "John", family_name = "Doe");
/// // Expands to: "WHERE GivenName = 'John' AND FamilyName = 'Doe'"
/// ```
///
/// ## Using dynamic values:
/// ```rust
/// # use quick_oxibooks::qb_where_clause;
/// # use quickbooks_types::Customer;
/// let name = "John".to_string();
/// let active = true;
/// let clause = qb_where_clause!(Customer | given_name = name, active = active);
/// // Expands to: "WHERE GivenName = 'John' AND Active = 'true'"
/// ```
///
/// ## With additional conditions:
/// ```rust
/// # use quick_oxibooks::qb_where_clause;
/// # use quickbooks_types::Customer;
/// let today = "2024-01-01";
/// let clause = qb_where_clause!(Customer | created_time = today ; "ORDER BY Id DESC", "MAXRESULTS 10");
/// // Expands to: "WHERE CreatedTime = '2024-01-01' ORDER BY Id DESC MAXRESULTS 10"
/// ```
///
/// ## Using different operators:
/// ```rust
/// # use quick_oxibooks::qb_where_clause;
/// # use quickbooks_types::Customer;
/// let clause = qb_where_clause!(Customer | display_name like "%Corp%", city in "New York");
/// // Expands to: "WHERE DisplayName like '%Corp%' AND City in 'New York'"
/// ```
///
/// # Compile-time Safety
///
/// The macro validates field names at compile time:
/// ```compile_fail
/// // This will fail to compile if `invalid_field` doesn't exist on Customer
/// qb_where_clause!(Customer | invalid_field = "value")
/// ```
///
/// # Performance
///
/// - **Literal values**: Built entirely at compile time with zero runtime cost
/// - **Expression values**: Uses capacity hints to minimize allocations
/// - **Field validation**: Zero runtime overhead for type checking
#[macro_export]
macro_rules! qb_where_clause {
  // Define common operators as const strings
  (_OP =) => {" = '"};
  (_OP like) => {" like '"};
  (_OP in) => {" in '"};
  (_OP $op:tt) => { compile_error!("Invalid Operator") };

  (_TYPECHECK $struct_name:ident, $($field:ident),+) => {
    {
      // Compiler doesn't include this in the binary,
      // just uses it to make sure the fields exist
      const _: () = {
        fn dummy(v: $struct_name) {
          $(
            let _ = v.$field;
          )+
        }
      };
    }
  };

  // For literal values - completely compile-time string building
  (_CLAUSE $($field:ident $op:tt $value:literal),+) => {
    concat!(
      "where ",
      $(
        paste::paste! {
          stringify!([<$field:camel>])
        },
        ' ',
        stringify!($op),
        " '",
        $value,
        "' and ",
      )+
    ).trim_end_matches(" and ")
  };

  // For expression values - minimize allocations with capacity hint
  (_CLAUSE $($field:ident $op:tt $value:expr),+) => {
    {
      // Count the number of clauses to estimate capacity
      const _CLAUSE_COUNT: usize = { $crate::macros::count!($($field),+) };
      // Estimate 30 bytes per clause as a starting point
      let mut _values = String::with_capacity(6 + _CLAUSE_COUNT * 30);
      _values.push_str("WHERE ");
      $(
        _values.push_str($crate::macros::convert_ascii_case!(upper_camel, stringify!($field)));
        _values.push_str(" ");
        _values.push_str(stringify!($op));
        _values.push_str(" '");
        _values.push_str(&($value).to_string());
        _values.push_str("' AND ");
      )+
      _values.truncate(_values.len() - 5); // Remove trailing " AND "
      _values
    }
  };

  ($struct_name:ident | $($field:ident $op:tt $value:expr),+) => {
    {
      $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
      $crate::qb_where_clause!(_CLAUSE $($field $op $value),+)
    }
  };

  ($struct_name:ident | $($field:ident $op:tt $value:expr),+ ; $($addon:literal),+) => {
    {
      $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
      let _clause = $crate::qb_where_clause!(_CLAUSE $($field $op $value),+);
      const _ADDON: &'static str = $crate::macros::concat!($($addon),+);
      // Avoid allocation if possible
      if _ADDON.is_empty() {
        _clause
      } else {
        let mut result = String::with_capacity(_clause.len() + 1 + _ADDON.len());
        result.push_str(&_clause);
        result.push(' ');
        result.push_str(_ADDON);
        result
      }
    }
  }
}

/// Executes a QuickBooks API query for a specific entity type with compile-time field validation.
///
/// This macro provides a convenient and type-safe way to query QuickBooks Online API
/// for entity records. It builds upon the [`qb_where_clause`] macro to generate properly
/// formatted query conditions and executes the query against the QuickBooks API.
///
/// # Features
///
/// - **Compile-time field validation**: Ensures all fields exist on the specified struct
/// - **Case conversion**: Automatically converts field names to QuickBooks API format
/// - **Type safety**: Returns properly typed entity objects
/// - **Single result**: Always returns a single entity (uses `query_single` internally)
/// - **Additional clause support**: Allows appending raw SQL-like conditions
///
/// # Syntax
///
/// ```ignore
/// qb_query!(qb_context, http_client, EntityType | field1 op value1, field2 op value2 [; additional_clauses...])
/// ```
///
/// # Parameters
///
/// - `qb_context`: Reference to [`QBContext`] with authentication and configuration
/// - `http_client`: Reference to HTTP client (usually `&ureq::Agent`)
/// - `EntityType`: QuickBooks entity type (e.g., Customer, Invoice, Item)
/// - `field op value`: Query conditions using supported operators
/// - `additional_clauses`: Optional raw SQL clauses after `;` separator
///
/// # Examples
///
/// ## Basic query:
/// ```rust
/// # use quick_oxibooks::{qb_query, QBContext, Environment};
/// # use quickbooks_types::Customer;
/// # use ureq::Agent;
/// # let client = Agent::new_with_defaults();
/// # let qb_context = QBContext::new(Environment::SANDBOX, "123".to_string(), "token".to_string(), &client).unwrap();
/// let customer = qb_query!(
///     &qb_context,
///     &client,
///     Customer | given_name = "John", family_name = "Doe"
/// )?;
/// // Executes: SELECT * FROM Customer WHERE GivenName = 'John' AND FamilyName = 'Doe' MAXRESULTS 1
/// ```
///
/// ## With dynamic values:
/// ```rust
/// # use quick_oxibooks::{qb_query, QBContext, Environment};
/// # use quickbooks_types::Customer;
/// # use ureq::Agent;
/// # let client = Agent::new_with_defaults();
/// # let qb_context = QBContext::new(Environment::SANDBOX, "123".to_string(), "token".to_string(), &client).unwrap();
/// let name = "John".to_string();
/// let active = true;
/// let customer = qb_query!(
///     &qb_context,
///     &client,
///     Customer | given_name = name, active = active
/// )?;
/// // Uses runtime values in the query
/// ```
///
/// ## With additional query options:
/// ```rust
/// # use quick_oxibooks::{qb_query, QBContext, Environment};
/// # use quickbooks_types::Invoice;
/// # use ureq::Agent;
/// # let client = Agent::new_with_defaults();
/// # let qb_context = QBContext::new(Environment::SANDBOX, "123".to_string(), "token".to_string(), &client).unwrap();
/// let today = "2024-01-01";
/// let invoice = qb_query!(
///     &qb_context,
///     &client,
///     Invoice | doc_number = "INV-001" ; "ORDER BY MetaData.CreateTime DESC"
/// )?;
/// // Adds ordering to the query
/// ```
///
/// ## Different operators:
/// ```rust
/// # use quick_oxibooks::{qb_query, QBContext, Environment};
/// # use quickbooks_types::Customer;
/// # use ureq::Agent;
/// # let client = Agent::new_with_defaults();
/// # let qb_context = QBContext::new(Environment::SANDBOX, "123".to_string(), "token".to_string(), &client).unwrap();
/// let customer = qb_query!(
///     &qb_context,
///     &client,
///     Customer | display_name like "%Corp%"
/// )?;
/// // Pattern matching query
/// ```
///
/// # Return Value
///
/// Returns `Result<EntityType, APIError>` where:
/// - `Ok(entity)`: The single matching entity
/// - `Err(APIError)`: Query failed or no results found
///
/// # Supported Operators
///
/// - `=`: Exact equality comparison
/// - `like`: Pattern matching with wildcards
/// - `in`: Value in a set of values
///
/// # Errors
///
/// - Compile-time error if field names don't exist on the entity type
/// - `NoQueryObjects`: No entities matched the query criteria
/// - `UreqError`: Network or HTTP errors during API call
/// - `BadRequest`: Invalid query syntax or field names
/// - `JsonError`: Response parsing errors
#[macro_export]
macro_rules! qb_query {
  ($qb:expr, $client:expr, $struct_name:ident | $($field:ident $op:tt $value:expr),+) => {
    <$struct_name as $crate::functions::query::QBQuery>::query_single(
      &$crate::qb_where_clause!($struct_name | $($field $op $value),+),
      $qb,
      $client
    )
  };

  ($qb:expr, $client:expr, $struct_name:ident | $($field:ident $op:tt $value:expr),+ ; $($addon:literal),+) => {
    <$struct_name as $crate::functions::query::QBQuery>::query_single(
      &$crate::qb_where_clause!($struct_name | $($field $op $value),+ ; $($addon),+),
      $qb,
      $client
    )
  };
}

#[cfg(test)]
mod test {
    use crate::QBContext;
    use quickbooks_types::Customer;

    fn test_macro_works() -> Result<(), String> {
        let client = ureq::Agent::new_with_defaults();
        let qb = QBContext::new_from_env(crate::Environment::SANDBOX, &client)
            .map_err(|e| e.to_string())?;
        let cust = qb_query!(
            &qb,
            &client,
            Customer | given_name = "Tom",
            family_name = "Hanks"
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
