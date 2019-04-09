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
use rocket_contrib::json::{Json, JsonValue};
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use lazy_static::lazy_static;
use mongodb::coll::Collection;

lazy_static!{
    static ref SETTINGS: data_types::Settings = data_types::Settings::new().unwrap();
    static ref DB_CL: Collection = database::connect_to_database();
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
fn register(registerform: Json<data_types::RegisterMessage>, auth_header: data_types::AuthHeader) -> JsonValue {
    match handlers::handle_register(auth_header.0, registerform.username, registerform.terms) {
        Err(e) => {
            dbg!(&e);
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

#[post("/verifyemail", format="json", data="<verifydata>")]
fn verify_email(verifydata: Json<data_types::VerifyEmailMessage>) -> JsonValue {
    match handlers::handle_verify(verifydata.email, verifydata.id) {
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

    rocket::ignite().mount("/",
    routes![
        home_page,
        handlerer,
        register,
        verify_email,
        test_post
    ]).launch();
}
