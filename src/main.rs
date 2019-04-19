#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate mongodb;

mod data_types;
mod utils;
mod handlers;
pub mod database;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};
use std::io;
use std::sync::Mutex;
use rocket_contrib::json::{Json, JsonValue};
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use lazy_static::lazy_static;
use mongodb::coll::Collection;
use std::net::SocketAddr;
use rocket::http::Cookies;
use redis::Connection;

lazy_static!{
    static ref SETTINGS: data_types::Settings = data_types::Settings::new().unwrap();
    static ref DB_CL: Collection = database::connect_to_database();
    static ref REDIS: Mutex<Connection> = database::connect_to_redis();
}

#[get("/")]
fn home_page() -> io::Result<NamedFile> {
    NamedFile::open("static/build/index.html")
}

#[get("/<file..>", rank = 10)]
fn handlerer(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/build/").join(file)).ok().or_else(|| NamedFile::open(Path::new("static/build/index.html")).ok())
}

#[post("/register", format="json", data="<registerform>")]
fn register(registerform: Json<data_types::RegisterMessage>, auth_header: data_types::AuthHeader, socket: SocketAddr) -> JsonValue {
    match handlers::handle_register(auth_header, registerform.username, registerform.terms, socket) {
        Err(e) => {
            match e {
                data_types::RegisterError::ExistsUsername => return json!({"status": "error", "message": "ExistsUsername"}),
                data_types::RegisterError::ExistsEmail => return json!({"status": "error", "message" : "ExistsEmail"}),
                data_types::RegisterError::IllegalCharacters => return json!({"status": "error", "message": "IllegalCharacters"}),
                data_types::RegisterError::BadLength => return json!({"status": "error", "message": "BadLength"}),
                data_types::RegisterError::Terms => return json!({"status": "error", "message": "Terms"}),
                data_types::RegisterError::Error  => return json!({"status": "error", "message": "unknown"}),
            }
        },
        Ok(_) => return json!({"status": "ok", "message": "VerifyEmail"})
    }
}

#[post("/signin")]
fn signin(auth_header: data_types::AuthHeader, socket: SocketAddr) -> JsonValue {
    match handlers::handle_signin(auth_header, socket) {
        Err(e) => match e {
            data_types::SignInError::Invalid => return json!({"status": "error", "message": "InvalidData"}),
            data_types::SignInError::NotVerified => return json!({"status": "error", "message": "NotVerified"}),
            data_types::SignInError::Token => return json!({"status": "error", "message": "ErrorToken"}),
            data_types::SignInError::Error => return json!({"status": "error", "message": "unknown"}),
        },
        Ok(token) => return json!({"status": "ok", "message": token})
    }
}

#[post("/verifysession")]
fn verify_session(cookies: Cookies, socket: SocketAddr) -> JsonValue {
    let token = cookies.get("token").unwrap().value();
    match utils::check_token(token, socket) {
        Ok(_) => return json!({"status": "ok", "message": ""}),
        Err(_) => return json!({"status": "error", "message": "invalidToken"})
    }
}

#[post("/verifyemail", format="json", data="<verifydata>")]
fn verify_email(verifydata: Json<data_types::VerifyEmailMessage>) -> JsonValue {
    match handlers::handle_verify_email(verifydata.email, verifydata.id) {
        Ok(_) => return json!({"status": "ok"}),
        Err(e) => match e {
            data_types::VerifyResult::Error => return json!({"status": "error"})
        }
    }
}

#[post("/test", format="json", data="<message>")]
fn test_post(message: Json<data_types::TestMessage>) -> JsonValue {
//    let mut stream = UnixStream::connect("/home/yknomeh/socket").unwrap();
//    stream.write_all(message.0.message.as_bytes()).unwrap();
    json!({"status": "ok"})
}

fn main() {
    if !Path::new("../server.toml").exists() {
        println!("No server config file");
        return;
    }

    { let _ = &DB_CL.namespace; } // Force initializing database
    { let _ = &REDIS.lock().unwrap().is_open(); } // Force initializing redis

    rocket::ignite().mount("/",
    routes![
        home_page,
        handlerer,
        signin,
        register,
        verify_session,
        verify_email,
        test_post
    ]).launch();
}
