use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;
use rocket::http::Status;
use super::utils;

#[derive(Serialize, Deserialize)]
pub struct TestMessage<'wtf> {
    pub message: &'wtf str
}

#[derive(Serialize, Deserialize)]
pub struct RegisterMessage<'a> {
    pub username: &'a str,
    pub terms: bool
}

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