pub use const_str::{concat, convert_ascii_case};

/// # qb_where_clause
///
/// Creates a SQL-like WHERE clause string for QuickBooks API queries with compile-time field validation.
///
/// This macro generates properly formatted WHERE clauses for QuickBooks Online API, converting
/// field names from snake_case to UpperCamelCase automatically, joining multiple conditions
/// with "AND", and properly escaping values.
///
/// ## Features
///
/// - **Compile-time field validation**: Ensures all fields exist on the specified struct
/// - **Case conversion**: Automatically converts field names to QuickBooks API format (UpperCamelCase)
/// - **Memory optimization**: Uses compile-time string building for literals and capacity hints for expressions
/// - **Additional clause support**: Allows appending raw SQL-like conditions
///
/// ## Usage
///
/// ### Basic WHERE clause:
/// ```rust
/// qb_where_clause!(Customer | given_name = "John", family_name = "Doe")
/// // Expands to: "WHERE GivenName = 'John' AND FamilyName = 'Doe'"
/// ```
///
/// ### Using dynamic values:
/// ```rust
/// let name = get_name();
/// qb_where_clause!(Customer | given_name = name, active = true)
/// // Expands to: "WHERE GivenName = '[name value]' AND Active = 'true'"
/// ```
///
/// ### With additional conditions:
/// ```rust
/// qb_where_clause!(Customer | created_at = today ; "ORDER BY Id DESC")
/// // Expands to: "WHERE CreatedAt = '[today value]' ORDER BY Id DESC"
/// ```
///
/// ## Supported operators
///
/// The macro supports the following operators:
/// - `=` (equals)
/// - `like` (pattern matching)
/// - `in` (value in set)
///
/// ## Expansion
///
/// For the pattern `qb_where_clause!(Struct | field1 = value1, field2 = value2)`:
///
/// 1. Validates that `field1` and `field2` exist on `Struct` (compile-time check)
/// 2. Converts field names to UpperCamelCase: `field1` → `Field1`
/// 3. Builds a WHERE clause string with proper formatting
/// 4. For literal values, builds strings at compile time
/// 5. For expression values, builds strings at runtime with capacity optimization
///
/// The `;` separator allows adding raw SQL fragments like `ORDER BY`, `LIMIT`, etc.
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
    $crate::macros::concat!(
      "where ",
      $(
        $crate::macros::convert_ascii_case!(upper_camel, stringify!($field)),
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

/// # qb_query
///
/// Executes a QuickBooks API query for a specific entity type with compile-time field validation.
///
/// This macro provides a convenient and type-safe way to query QuickBooks Online API
/// for entity records. It builds upon the `qb_where_clause` macro to generate properly
/// formatted query conditions and executes the query against the QuickBooks API.
///
/// ## Features
///
/// - **Compile-time field validation**: Ensures all fields exist on the specified struct
/// - **Case conversion**: Automatically converts field names to QuickBooks API format
/// - **Type safety**: Returns properly typed entity objects
/// - **Additional clause support**: Allows appending raw SQL-like conditions like ORDER BY, LIMIT, etc.
///
/// ## Usage
///
/// ### Basic query:
/// ```rust
/// let customer = qb_query!(
///     &qb_context,
///     &http_client,
///     Customer | given_name = "John", family_name = "Doe"
/// )?;
/// // Executes a query to find Customer where GivenName = 'John' AND FamilyName = 'Doe'
/// ```
///
/// ### With dynamic values:
/// ```rust
/// let name = get_name();
/// let customer = qb_query!(
///     &qb_context,
///     &http_client,
///     Customer | given_name = name, active = true
/// )?;
/// // Executes a query with runtime values
/// ```
///
/// ### With additional query options:
/// ```rust
/// let customers = qb_query!(
///     &qb_context,
///     &http_client,
///     Customer | created_at = today ; "ORDER BY Id DESC MAXRESULTS 10"
/// )?;
/// // Adds ordering and result limits to the query
/// ```
///
/// ## Supported operators
///
/// The macro supports the same operators as `qb_where_clause`:
/// - `=` (equals)
/// - `like` (pattern matching)
/// - `in` (value in set)
///
/// ## Expansion
///
/// For the pattern `qb_query!($qb, $client, Struct | field1 = value1, field2 = value2)`:
///
/// 1. Generates a WHERE clause using `qb_where_clause!`
/// 2. Calls `<Struct as QBQuery>::query_single()` with the generated WHERE clause
/// 3. Passes the QuickBooks context and HTTP client to handle the API request
/// 4. Returns the result of the query as a `Result<Struct, Error>`
///
/// When additional clauses are provided after `;`, they are appended to the query string.
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
        let client = reqwest::Client::new();
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
