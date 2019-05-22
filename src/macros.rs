macro_rules! outcome {
    ({"status": $status:expr, "message": $message:expr}) => {
            return Ok(format!("{{\"status\": \"{}\", \"message\": \"{}\"}}", $status, $message))
    };
}