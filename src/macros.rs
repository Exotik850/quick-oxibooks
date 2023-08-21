pub use const_str::{concat, convert_ascii_case};

#[macro_export]
macro_rules! qb_where_clause {
    (_TYPECHECK $struct_name:ident, $($field:ident),+) => {
        {
            const _: () = {
                fn dummy(v: $struct_name) {
                    $(
                        let _ = v.$field;
                    )+
                }
            };
        }
    };

    (_CLAUSE $($field:ident: $value:literal),+) => {
        {
            $crate::macros::concat!(
                "where ",
                $(
                    $crate::macros::convert_ascii_case!(upper_camel, stringify!($field)),
                    " = '",
                    $value,
                    "' AND ",
                )+
            ).trim_end_matches(" AND ")
        }
    };
    
    (_CLAUSE $($field:ident: $value:expr),+) => {
        {
            let mut _values = String::new();
            $(
                _values.push_str("WHERE ");
                _values.push_str($crate::macros::convert_ascii_case!(upper_camel, stringify!($field)));
                _values.push_str(" = '");
                _values += &($value).to_string();
                _values.push_str("' AND ".into());
            )+

            let _final_length = _values.len() - 5;
            _values.truncate(_final_length);
            _values
        }
    };

    ($struct_name:ident | $($field:ident: $value:expr),+) => {
        {
            $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
            $crate::qb_where_clause!(_CLAUSE $($field : $value),+)
        }
    };

    ($struct_name:ident | $($field:ident : $value:expr),+ ; $($addon:literal),+) => {
        {
            $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
            let _CLAUSE = $crate::qb_where_clause!(_CLAUSE $($field : $value),+)
            let _ADDON = $crate::macros::concat!($($addon),+);
            format!("{_CLAUSE} {_ADDON}")
        }
    }
}

fn _test() {
    use quickbooks_types::Customer;
    let _tes = format!("{}", 10);
    let _query = {
        crate::qb_where_clause!(_TYPECHECK Customer,id);
        crate::qb_where_clause!(_CLAUSE id:20u8)
    };
}