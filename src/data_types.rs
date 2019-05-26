// use rocket::request::{self, Request, FromRequest};
// use rocket::Outcome;
// use rocket::http::Status;
use super::utils;
use config::{ConfigError, Config};
use std::path::Path;
use actix_web::{HttpRequest, http::header::HeaderMap, http::header::Header};

#[derive(Serialize, Deserialize)]
pub struct TestMessage {
    pub message: String
}

// Json from register form
#[derive(Deserialize)]
pub struct RegisterMessage {
    pub username: String,
    pub terms: bool
}

// Json for verify function
#[derive(Serialize, Deserialize)]
pub struct VerifyEmailMessage {
    pub email: String,
    pub id: String
}

// Authorization token
#[derive(Debug)]
pub struct AuthHeader {
    pub email: String,
    pub password: String,
    pub confirm_password: String
}

 #[derive(Debug)]
 pub enum AuthHeaderError {
     Missing,
     Invalid
 }

 impl AuthHeader {
     pub fn new(request: &HttpRequest) -> Result<AuthHeader, AuthHeaderError> {
         let header = utils::get_auth_header(request.headers());

         match header {
             Ok(auth_header) => {
                 if utils::is_auth_header_valid(auth_header) {
                     return Ok(utils::get_creds(auth_header));
                 } else {
                     return Err(AuthHeaderError::Invalid);
                 }
             },
             Err(err) => return Err(err)
         }
     }
 }

// Settings file
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub secret: Secret,
    pub redis: Redis,
    pub auth: AuthRequirements,
    pub email: Email,
    pub smtp: Smtp
}

#[derive(Debug, Deserialize)]
pub struct Secret {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    pub user: String,
    pub password: String,
    pub address: String,
    pub port: u16
}

#[derive(Debug, Deserialize)]
pub struct AuthRequirements {
    pub username_len_min: usize,
    pub username_len_max: usize,
    pub password_len_min: usize,
    pub password_len_max: usize,
    pub email_len_min: usize,
    pub email_len_max: usize
}

#[derive(Debug, Deserialize)]
pub struct Email {
    pub noreply: String,
    pub support: String
}

#[derive(Debug, Deserialize)]
pub struct Smtp {
    pub username: String,
    pub password: String,
    pub server: String
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        if !Path::new("../server.toml").exists() {
            println!("No server config file");
        }

        settings.merge(config::File::with_name("../server"))?;

        settings.try_into()
    }
}

// Register errors
#[derive(Debug)]
pub enum RegisterError {
    ExistsUsername,
    ExistsEmail,
    IllegalCharacters,
    BadLength,
    Terms,
    Error
}

// SignIn errors
#[derive(Debug)]
pub enum SignInError {
    NotVerified,
    Invalid,
    Token,
    Error
}

// Verification results
#[derive(Debug)]
pub enum VerifyResult {
    Error
}

// JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub uid: String,
    pub ip: String,
    pub exp: String
}