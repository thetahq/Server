#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

mod data_types;
mod utils;
mod handlers;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};
use std::io;
use rocket_contrib::json::{Json, JsonValue};
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::sync::RwLock;
use config::Config;

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
    handlers::handle_register(auth_header.0, registerform.username, registerform.terms);

    json!({"status": "ok"})
}

#[post("/test", format="json", data="<message>")]
fn test_post(message: Json<data_types::TestMessage>) -> JsonValue {
//    let mut stream = UnixStream::connect("/home/yknomeh/socket").unwrap();
//    stream.write_all(message.0.message.as_bytes()).unwrap();

    json!({"status": "ok"})
}

fn main() {
    if cfg!(target_os = "windows") {
        println!("We don't use windows here.");
        return;
    }

    if !Path::new("../server.toml").exists() {
        println!("No server config file");
        return;
    }
    let config = data_types::Settings::new().unwrap();

    dbg!(config);

    rocket::ignite().mount("/",
    routes![
        home_page,
        handlerer,
        register,
        test_post
    ]).launch();
}
