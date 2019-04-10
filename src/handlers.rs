use std::str;
use bson;
use mongodb::{Bson, doc};
use super::SETTINGS;
use super::data_types;
use super::utils;
use super::DB_CL;
use jsonwebtoken::{encode, Header};
use chrono::prelude::Utc;
use chrono::Duration;
use std::net::SocketAddr;
use sha3::{Digest, Sha3_256};

pub fn handle_register(header: data_types::AuthHeader, username: &str, terms: bool, socket: SocketAddr) -> Result<(), data_types::RegisterError> {
    // TODO check if creds does not contain illegal characters
    // TODO optimise it

    if header.email.len() < SETTINGS.auth.email_len_min || header.email.len() > SETTINGS.auth.email_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if header.password.len() < SETTINGS.auth.password_len_min || header.password.len() > SETTINGS.auth.password_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if username.len() < SETTINGS.auth.username_len_min || username.len() > SETTINGS.auth.username_len_max {
        return Err(data_types::RegisterError::BadLength);
    }

    if !terms {
        return Err(data_types::RegisterError::Terms);
    }

    let check_username_data = doc! {
        "username": username
    };

    let mut cursor = DB_CL.find(Some(check_username_data.clone()), None).ok().expect("Failed while executing find");

    match cursor.next() {
        Some(Ok(_doc)) => return Err(data_types::RegisterError::ExistsUsername),
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => {}
    }

    let check_mail_data = doc! {
        "email": header.email.to_owned()
    };

    let mut cursor = DB_CL.find(Some(check_mail_data.clone()), None).ok().expect("Failed while executing find");

    match cursor.next() {
        Some(Ok(_doc)) => return Err(data_types::RegisterError::ExistsEmail),
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => {}
    }

    let pass = Sha3_256::digest(header.password.as_bytes());

    let doc = doc! {
        "username": username,
        "email": header.email.to_owned(),
        "password": format!("{:x}", pass),
        "ips": [socket.ip().to_string()],
        "verified": false
    };

    DB_CL.insert_one(doc.clone(), None).ok().expect("Failed to insert document.");

    let mut cursor = DB_CL.find(Some(doc.clone()), None).ok().expect("Failed while executing find");

    match cursor.next() {
        Some(Ok(doc)) => match doc.get("_id") {
            Some(&Bson::ObjectId(ref id)) => {
                // @todo sends gets old ID (idk why. It worked before). Database does not refresh or something
                utils::send_registration_mail(header.email.to_owned(), username, id.to_string());
            },
            _ => return Err(data_types::RegisterError::Error)
        },
        Some(Err(_)) => return Err(data_types::RegisterError::Error),
        None => return Err(data_types::RegisterError::Error)
    }

    Ok(())
}

pub fn handle_signin(header: data_types::AuthHeader, socket: SocketAddr) -> Result<String, data_types::SignInError> {
    let pass = Sha3_256::digest(header.password.as_bytes());

    let user_data = doc! {
        "email": header.email.to_owned(),
        "password": format!("{:x}", pass)
    };

    match DB_CL.find_one(Some(user_data.clone()), None) {
        Ok(doc) => match doc {
            Some(data) => {
                match data.get("verified") {
                    Some(&Bson::Boolean(ref verified)) => {
                        if !verified {
                            return Err(data_types::SignInError::NotVerified);
                        }

                        match DB_CL.update_one(data.clone(), doc! {"$push":{"ips": socket.ip().to_string()}}, None).ok() {
                            Some(res) => {
                                if res.matched_count != 1 && res.modified_count != 1 {
                                    return Err(data_types::SignInError::Error);
                                }
                            },
                            None => {
                                return Err(data_types::SignInError::Error);
                            }
                        }
                    }
                    _  => return Err(data_types::SignInError::Error)
                }

                match data.get("_id") {
                    Some(&Bson::ObjectId(ref id)) => {
                        let date = Utc::now() + Duration::weeks(1);

                        let claims = data_types::Claims {
                            uid: id.to_string(),
                            ip: socket.ip().to_string(),
                            exp: date.format("%Y-%m-%d").to_string()
                        };

                        let token = encode(&Header::default(), &claims, SETTINGS.secret.key.as_ref());

                        match token {
                            Ok(tok)=> return Ok(tok),
                            Err(_) => return Err(data_types::SignInError::Token)
                        }

                    },
                    _ => return Err(data_types::SignInError::Error)
                }
            },
            None => return Err(data_types::SignInError::Invalid)
        },
        Err(_) => return Err(data_types::SignInError::Invalid)
    }
}

pub fn handle_verify_email(email: &str, id: &str) -> Result<(), data_types::VerifyResult> {
    let doc = doc! {
        "_id": bson::oid::ObjectId::with_string(id).unwrap(),
        "email": bson::Bson::String(email.to_string()),
        "verified": false
    };

    let update = DB_CL.update_one(doc.clone(), doc! {"$set":{"verified": true}}, None).ok();

    match update {
        Some(res) => {
            if res.matched_count != 1 && res.modified_count != 1 {
                return Err(data_types::VerifyResult::Error);
            }
        },
        None => {
            println!("verification failed");
            return Err(data_types::VerifyResult::Error);
        }
    }

    Ok(())
}