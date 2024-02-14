#[macro_export]
macro_rules! response {
    ( $map:expr ) => {
        (rocket::http::Status::from_code(200).unwrap(), rocket::serde::json::json!($map))
    };
    ( $status_code:expr, $msg:expr $(, $key:expr => $val:expr )* ) => {
        (rocket::http::Status::from_code($status_code).unwrap(), rocket::serde::json::json!({
                "message": $msg,
                "error_code": $status_code,
                $( $key: $val, )*
            }))
    };
    ( $msg:expr $(, $key:expr => $val:expr )+ ) => {
        (rocket::http::Status::from_code(200).unwrap(), rocket::serde::json::json!({
                "message": $msg,
                $( $key: $val, )*
            }))
    };
    ( $($key:expr => $val:expr ),* ) => {
        (rocket::http::Status::from_code(200).unwrap(), rocket::serde::json::json!({
                $( $key: $val, )*
            }))
    };
}
