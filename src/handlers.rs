 use std::str;
 use super::SETTINGS;
 use super::REDIS;
 use super::data_types;
 use super::utils;
 use jsonwebtoken::{encode, Header};
 use chrono::prelude::Utc;
 use chrono::Duration;
 use std::net::SocketAddr;
 use sha3::{Digest, Sha3_256};
 use redis::Commands;
 use uuid::Uuid;

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

     let red = REDIS.lock().unwrap();
     let exists_username: bool = red.exists(format!("username:{}", username)).unwrap();

     if exists_username {
         return Err(data_types::RegisterError::ExistsUsername);
     }

     let exists_email: bool = red.exists(format!("email:{}", header.email.to_owned())).unwrap();

     if exists_email {
         return Err(data_types::RegisterError::ExistsEmail);
     }

     let pass = Sha3_256::digest(header.password.as_bytes());

     let mut new_uuid = "".to_string();

     loop {
         new_uuid = Uuid::new_v4().to_string();
         let uid_exists: bool = red.exists(format!("user:{}", new_uuid.to_owned())).unwrap();

         if !uid_exists {
             break;
         }
     }

     let reg_result: Result<(),redis::RedisError> = red.hset_multiple(format!("user:{}", new_uuid.to_owned()), &[
         ("username", username),
         ("email", &header.email.to_owned()),
         ("password", format!("{:x}", pass).as_str()),
         ("verified", "false")
     ]);

     match reg_result {
         Ok(_) => {},
         Err(_err) => return Err(data_types::RegisterError::Error)
     }

     let ips_set: Result<(),redis::RedisError> = red.sadd(format!("ips:{}", new_uuid.to_owned()), &socket.ip().to_string());

     match ips_set {
         Ok(_) => {},
         Err(_err) => return Err(data_types::RegisterError::Error)
     }

     let ref_username: Result<(),redis::RedisError> = red.sadd(format!("username:{}", username), new_uuid.to_owned());

     match ref_username {
         Ok(_) => {},
         Err(_err) => return Err(data_types::RegisterError::Error)
     }

     let ref_email: Result<(),redis::RedisError> = red.sadd(format!("email:{}", header.email.to_owned()), new_uuid.to_owned());

     match ref_email {
         Ok(_) => {},
         Err(_err) => return Err(data_types::RegisterError::Error)
     }

     utils::send_registration_mail(header.email.to_owned(), username, new_uuid);

     Ok(())
 }

// pub fn handle_signin(header: data_types::AuthHeader, socket: SocketAddr) -> Result<String, data_types::SignInError> {
//     let pass = Sha3_256::digest(header.password.as_bytes());

//     let red = REDIS.lock().unwrap();
    
//     let user_id: Vec<String> = red.sinter(format!("email:{}", header.email.to_owned())).unwrap();
//     if user_id.len() == 0 {
//         return Err(data_types::SignInError::Invalid);
//     }

//     let user_password: String = red.hget(format!("user:{}", user_id[0]), "password").unwrap();
//     if format!("{:x}", pass) != user_password {
//         return Err(data_types::SignInError::Invalid);
//     }

//     let user_verified: bool = red.hget(format!("user:{}", user_id[0]), "verified").unwrap();
//     if !user_verified {
//         return Err(data_types::SignInError::NotVerified);
//     }

//     let ips_set: Result<(),redis::RedisError> = red.sadd(format!("ips:{}", user_id[0]), &socket.ip().to_string());
//     match ips_set {
//         Ok(_) => {},
//         Err(_err) => return Err(data_types::SignInError::Error)
//     }

//     let date = Utc::now() + Duration::weeks(1);

//     let claims = data_types::Claims {
//         uid: user_id[0].to_owned(),
//         ip: socket.ip().to_string(),
//         exp: date.format("%Y-%m-%d").to_string()
//     };

//     let token = encode(&Header::default(), &claims, SETTINGS.secret.key.as_ref());

//     match token {
//         Ok(tok)=> return Ok(tok),
//         Err(_) => return Err(data_types::SignInError::Token)
//     }
// }

// pub fn handle_verify_email(email: &str, id: &str) -> Result<(), data_types::VerifyResult> {
//     let red = REDIS.lock().unwrap();
//     let user_email: Vec<String> = red.sinter(format!("email:{}", email)).unwrap();

//     if user_email.len() == 0 {
//         return Err(data_types::VerifyResult::Error);
//     }

//     if user_email[0] != id {
//         return Err(data_types::VerifyResult::Error);
//     }

//     let set_result: Result<(),redis::RedisError> = red.hset(format!("user:{}", id), "verified", "true");

//     match set_result {
//         Ok(_) => {},
//         Err(_err) => return Err(data_types::VerifyResult::Error)
//     }

//     Ok(())
// }