use std::str;
use mongodb::{Bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use super::SETTINGS;

pub fn handle_register(header: &str, username: &str, terms: bool) {
    let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
    let decoded= str::from_utf8(&bytes).unwrap_or_default();

    let creds: Vec<&str> = decoded.split(":").collect();
    // TODO Check if terms are true and if username is longer than 5 characters and less than 15(?)
    // TODO JWT
    // TODO hash password
    // TODO Check if username or email exists

    let client = Client::connect("localhost", 27017).unwrap();
    let db = client.db("admin");
    let auth_result = db.auth(&SETTINGS.mongo.user, &SETTINGS.mongo.password);
    let col = client.db("thetahq").collection("users");

    let doc = doc! {
        "username": username,
        "email": creds[0],
        "password": "HASHHASH"
    };

    col.insert_one(doc.clone(), None).ok().expect("Failed to insert document.");
}