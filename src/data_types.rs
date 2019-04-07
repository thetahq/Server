use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;
use rocket::http::Status;
use super::utils;
use config::{ConfigError, Config};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct TestMessage<'wtf> {
    pub message: &'wtf str
}

// Json from register form
#[derive(Serialize, Deserialize)]
pub struct RegisterMessage<'a> {
    pub username: &'a str,
    pub terms: bool
}

// Json for verify function
#[derive(Serialize, Deserialize)]
pub struct VerifyEmailMessage<'a> {
    pub email: &'a str,
    pub id: &'a str
}

// Authorization token
#[derive(Debug)]
pub struct AuthHeader<'a>(pub &'a str);

#[derive(Debug)]
pub enum AuthHeaderError {
    BadCount,
    Missing,
    Invalid
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthHeader<'a> {
    type Error = AuthHeaderError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = request.headers().get("Authorization").collect();
        match headers.len() {
            0 => Outcome::Failure((Status::BadRequest, AuthHeaderError::Missing)),
            1 if utils::is_auth_header_valid(headers[0]) => Outcome::Success(AuthHeader(headers[0])),
            1 => Outcome::Failure((Status::BadRequest, AuthHeaderError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, AuthHeaderError::BadCount)),
        }
    }
}

// Settings file
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub secret: Secret,
    pub mongo: Mongo,
    pub auth: AuthRequirements,
    pub email: Email,
    pub smtp: Smtp
}

#[derive(Debug, Deserialize)]
pub struct Secret {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct Mongo {
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

// Verification results
#[derive(Debug)]
pub enum VerifyResult {
    Error
}

// JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub uid: String,
    pub exp: String
}