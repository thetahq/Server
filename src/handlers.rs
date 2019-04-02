use std::str;

pub fn handle_register(header: &str, username: &str, terms: bool) {
    let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
    let decoded= str::from_utf8(&bytes).unwrap_or_default();

    let creds: Vec<&str> = decoded.split(":").collect();
    // TODO Check if terms are true and if username is longer than 5 characters and less than 15(?)
    // TODO JWT

    println!("{:#?}", creds);
}