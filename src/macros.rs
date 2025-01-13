pub use const_str::{concat, convert_ascii_case};

#[macro_export]
macro_rules! qb_where_clause {
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

    (_CLAUSE $($field:ident $op:tt $value:literal),+) => {
        {
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
        }
    };

    (_CLAUSE $($field:ident $op:tt $value:expr),+) => {
        {
            let mut _values = String::from("WHERE ");
            $(
                _values += $crate::macros::convert_ascii_case!(upper_camel, stringify!($field));
                _values += " ";
                _values += stringify!($op);
                _values += " '";
                _values += &($value).to_string();
                _values += "' AND ";
            )+
            _values.truncate(_values.len() - 5);
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
            format!("{_clause} {_ADDON}")
        }
    }
}

#[macro_export]
macro_rules! qb_query {
    ($qb:expr, $client:expr, $struct_name:ident | $($field:ident $op:tt $value:expr),+) => {
        $crate::functions::qb_query_single::<$struct_name>(
          &$crate::qb_where_clause!($struct_name | $($field $op $value),+),
          $qb,
          $client
        ).await
    };

    ($qb:expr, $client:expr, $struct_name:ident | $($field:ident $op:tt $value:expr),+ ; $($addon:literal),+) => {
        $crate::functions::qb_query_single::<$struct_name>(
          &$crate::qb_where_clause!($struct_name | $($field $op $value),+ ; $($addon),+),
          $qb,
          $client
        ).await
    };
}

// #[cfg(test)]
// mod test {
//     use quickbooks_types::Customer;

//     #[tokio::test]
//     async fn test_macro_works() -> Result<(), String> {
//         let client = reqwest::Client::new();
//         let cust = qb_query!(
//             &qb,
//             &client,
//             Customer | given_name = "Tom",
//             family_name = "Hanks"
//         )
//         .map_err(|e| e.to_string())?;
//         Ok(())
//     }
// }
