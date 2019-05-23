use super::data_types;

macro_rules! outcome {
    ({"status": $status:expr, "message": $message:expr}) => {
        return Ok(format!("{{\"status\": \"{}\", \"message\": \"{}\"}}", $status, $message))
    };
}

#[macro_export]
macro_rules! match_errors {
    (what = $what:expr, source = $source:ident, $($error:ident),*) => {
        $(
            use crate::data_types::$source::$error;
        )*

        match $what {
            $(
                data_types::$source::$error => outcome!{{"status": "error", "message": format!("{}", stringify!($error))}},
            )*
        }
    };
}