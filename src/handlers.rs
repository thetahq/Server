use std::str;
use mongodb::{Bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use super::SETTINGS;
use super::data_types;
use jsonwebtoken::{encode, Header};

pub fn handle_register(header: &str, username: &str, terms: bool) -> Result<String, data_types::RegisterError> {
    let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
    let decoded= str::from_utf8(&bytes).unwrap_or_default();

    let creds: Vec<&str> = decoded.split(":").collect();
    // TODO JWT
    // TODO hash password
    // TODO check if creds does not contain illegal characters
    // TODO optimise it

    if creds[0].len() < SETTINGS.auth.email_len_min || creds[0].len() > SETTINGS.auth.email_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if creds[1].len() < SETTINGS.auth.password_len_min || creds[1].len() > SETTINGS.auth.password_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if username.len() < SETTINGS.auth.username_len_min || username.len() > SETTINGS.auth.username_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if !terms {
        return Err(data_types::RegisterError::Terms);
    }

    let client = Client::connect(&SETTINGS.mongo.address.to_string(), SETTINGS.mongo.port).unwrap();
    let db = client.db("admin");
    let _auth_result = db.auth(&SETTINGS.mongo.user, &SETTINGS.mongo.password);
    let col = client.db("thetahq").collection("users");

    let check_username_data = doc! {
        "username": username
    };

    let mut cursor = col.find(Some(check_username_data.clone()), None).ok().expect("Failed while executing find");

    match cursor.next() {
        Some(Ok(_doc)) => return Err(data_types::RegisterError::ExistsUsername),
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => {}
    }

    let check_mail_data = doc! {
        "email": creds[0]
    };

    let mut cursor = col.find(Some(check_mail_data.clone()), None).ok().expect("Failed while executing find");

    match cursor.next() {
        Some(Ok(_doc)) => return Err(data_types::RegisterError::ExistsEmail),
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => {}
    }

    let doc = doc! {
        "username": username,
        "email": creds[0],
        "password": "HASHHASH"
    };

    col.insert_one(doc.clone(), None).ok().expect("Failed to insert document.");

    let mut cursor = col.find(Some(doc.clone()), None).ok().expect("Failed while executing find");

    let mut claims: data_types::Claims;

    match cursor.next() {
        Some(Ok(doc)) => match doc.get("_id") {
            Some(&Bson::ObjectId(ref id)) => {
                claims = data_types::Claims {
                    uid: id.to_string(),
                    exp: 10000000
                }
            },
            _ => return Err(data_types::RegisterError::Error)
        },
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => return Err(data_types::RegisterError::Error)
    }

    let token = match encode(&Header::default(), &claims, SETTINGS.secret.key.as_ref()) {
        Ok(tok) => tok,
        Err(_err) => panic!("Token error")
    };

    Ok(token)
}