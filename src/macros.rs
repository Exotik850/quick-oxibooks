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

    (_CLAUSE $($field:ident: $value:expr),+) => {
        {
            $crate::macros::concat!(
                "where ",
                $(
                    $crate::macros::convert_ascii_case!(upper_camel, stringify!($field)),
                    " = '",
                    stringify!($value),
                    "' AND ",
                )+
            )
        }
    };

    ($struct_name:ident | $($field:ident: $value:expr),+) => {
        {
            $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
            $crate::qb_where_clause!(_CLAUSE $($field : $value),+)
            .trim_end_matches(" AND ")
        }
    };

    ($struct_name:ident | $($field:ident : $value:expr),+ ; $($addon:literal),+) => {
        {
            $crate::qb_where_clause!(_TYPECHECK $struct_name, $($field),+);
            let _CLAUSE = $crate::qb_where_clause!(_CLAUSE $($field : $value),+)
                .trim_end_matches(" AND ");
            let _ADDON = $crate::macros::concat!($($addon),+);
            format!("{_CLAUSE} {_ADDON}")
        }
    }

}
