use std::str;

pub fn is_auth_header_valid(header: &str) -> bool {
    let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
    let decoded: &str = str::from_utf8(&bytes).unwrap_or_default();

    let creds: Vec<&str> = decoded.split(":").collect();

    if creds.len() != 3 {
        return false;
    }

    if !creds[0].contains(".") || !creds[0].contains("@") {
        return false
    }

    if creds[1] != creds[2] {
        return false;
    }

    true
}