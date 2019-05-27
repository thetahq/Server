#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;
mod data_types;
mod utils;
mod handlers;
mod mw;
pub mod database;

use std::path::{Path, PathBuf};
use std::io;
use std::sync::Mutex;
use std::os::unix::net::UnixStream;
use lazy_static::lazy_static;
use redis::Connection;
use actix_web::{get, post, web, App, HttpServer, Result, HttpMessage, HttpRequest, middleware::Compress};
use actix_files as fs;
use actix_web::web::Json;

lazy_static! {
    static ref SETTINGS: data_types::Settings = data_types::Settings::new().unwrap();
    static ref REDIS: Mutex<Connection> = database::connect_to_redis();
}

#[get("/")]
fn home_page() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/build/index.html")?)
}

#[get("/{file:.*}")]
fn handlerer(file: web::Path<String>) -> Result<fs::NamedFile> {
    let req_file = fs::NamedFile::open(Path::new("static/build/").join(file.to_string()));

    match req_file {
        Ok(f) => return Ok(f),
        Err(_) => return Ok(fs::NamedFile::open("static/build/index.html")?)
    }
}

#[post("/register")]
fn register(req: HttpRequest, register_form: Json<data_types::RegisterMessage>) -> Result<String> {
    let header = data_types::AuthHeader::new(&req);
    match header {
        Ok(auth_header) => {
            match handlers::handle_register(auth_header, register_form.username.as_str(), register_form.terms, req.peer_addr().unwrap()) {
                Err(e) => {
                    match_errors! {
                        what = e, source = RegisterError, Terms, BadLength, ExistsUsername, ExistsEmail, Error, IllegalCharacters
                    }
                }
                Ok(_) => outcome! {{"status": "ok", "message": "VerifyEmail"}}
            }
        }
        Err(err) => {
            match_errors! {
                what = err, source = AuthHeaderError, Missing, Invalid
            }
        }
    }


    outcome! {{"status": "error", "message": "unknown"}}
}

 #[post("/signin")]
 fn signin(req: HttpRequest) -> Result<String> {
     // @todo check if not already signed in
     let header = data_types::AuthHeader::new(&req);
     match header {
         Ok(auth_header) => {
             match handlers::handle_signin(auth_header, req.peer_addr().unwrap()) {
                 Err(e) => {
                     match_errors! {
                        what = e, source = SignInError, Invalid, NotVerified, Token, Error
                    }
                 }
                 Ok(token) => outcome! {{"status": "ok", "message": token}}
             }
         }
         Err(err) => {
             match_errors! {
                what = err, source = AuthHeaderError, Missing, Invalid
            }
         }
     }
 }

 #[post("/verifysession")]
 fn verify_session(req: HttpRequest) -> Result<String> {
     match req.cookie("token") {
         Some(cookie) => match utils::check_token(cookie.value(), req.peer_addr().unwrap()) {
             Ok(_) => outcome! {{"status": "ok", "message": ""}},
             Err(_) => outcome! {{"status": "error", "message": "invalidToken"}}
         },
         None => outcome! {{"status": "error", "message": "noToken"}}
     }
 }

 #[post("/verifyemail")]
 fn verify_email(verify_data: Json<data_types::VerifyEmailMessage>) -> Result<String> {
     match handlers::handle_verify_email(verify_data.email.as_str(), verify_data.id.as_str()) {
         Ok(_) => outcome! {{"status": "ok", "message": "verified"}},
         Err(e) => {
             match_errors! {
                what = e, source = VerifyResult, Error
             }
         }
     }
 }

 #[post("/test")]
 fn test_post(message: Json<data_types::TestMessage>) -> Result<String> {
     //    let mut stream = UnixStream::connect("/home/yknomeh/socket").unwrap();
 //    stream.write_all(message.0.message.as_bytes()).unwrap();
     outcome! {{"status": "ok"}}
 }

fn main() {
    //@todo middleware logs for every request
    if !Path::new("../server.toml").exists() {
        println!("No server config file");
        return;
    }

    { let _ = &REDIS.lock().unwrap().is_open(); } // Force initializing redis

    env_logger::init();
    utils::log("Starting the server");

    HttpServer::new(
        || App::new()
            .wrap(Compress::default())
            .wrap(mw::Logging)
            .service(home_page)
            .service(handlerer)
            .service(register)
            .service(signin)
            .service(verify_email)
            .service(verify_session)
            .service(test_post)
    ).bind("127.0.0.1:8000").expect("Could not bind to 8000 port").run();
}
