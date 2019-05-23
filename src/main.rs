#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;
mod data_types;
mod utils;
mod handlers;
pub mod database;

use std::path::{Path, PathBuf};
use std::io;
use std::sync::Mutex;
use std::os::unix::net::UnixStream;
use lazy_static::lazy_static;
use redis::Connection;
use actix_web::{get, post, web, App, HttpServer, Result, HttpRequest, middleware, http::header::HeaderMap, http::header::Header};
use actix_files as fs;
use serde_json::json;
use actix_web::web::Json;

lazy_static! {
    static ref SETTINGS: data_types::Settings = data_types::Settings::new().unwrap();
    static ref REDIS: Mutex<Connection> = database::connect_to_redis();
}

#[get("/")]
fn home_page() -> Result<fs::NamedFile> {
    utils::log("GET -> /");
    Ok(fs::NamedFile::open("static/build/index.html")?)
}

#[get("/{file:.*}")]
fn handlerer(file: web::Path<String>) -> Result<fs::NamedFile> {
    utils::log(format!("GET -> {}", file.to_string()).as_str());

    let req_file = fs::NamedFile::open(Path::new("static/build/").join(file.to_string()));

    match req_file {
        Ok(f) => return Ok(f),
        Err(_) => return Ok(fs::NamedFile::open("static/build/index.html")?)
    }
}

#[post("/register")]
fn register(req: HttpRequest, register_form: web::Json<data_types::RegisterMessage>) -> Result<String> {
    utils::log("POST -> /register");

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
     utils::log("POST -> /signin");

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

// #[post("/verifysession")]
// fn verify_session(cookies: Cookies, socket: SocketAddr) -> JsonValue {
//     let token = cookies.get("token").unwrap().value();
//     match utils::check_token(token, socket) {
//         Ok(_) => return json!({"status": "ok", "message": ""}),
//         Err(_) => return json!({"status": "error", "message": "invalidToken"})
//     }
// }

// #[post("/verifyemail", format="json", data="<verifydata>")]
// fn verify_email(verifydata: Json<data_types::VerifyEmailMessage>) -> JsonValue {
//     match handlers::handle_verify_email(verifydata.email, verifydata.id) {
//         Ok(_) => return json!({"status": "ok"}),
//         Err(e) => match e {
//             data_types::VerifyResult::Error => return json!({"status": "error"})
//         }
//     }
// }

// #[post("/test", format="json", data="<message>")]
// fn test_post(message: Json<data_types::TestMessage>) -> JsonValue {
// //    let mut stream = UnixStream::connect("/home/yknomeh/socket").unwrap();
// //    stream.write_all(message.0.message.as_bytes()).unwrap();
//     json!({"status": "ok"})
// }

fn main() {
    if !Path::new("../server.toml").exists() {
        println!("No server config file");
        return;
    }

    { let _ = &REDIS.lock().unwrap().is_open(); } // Force initializing redis

    utils::log("Starting the server");

    HttpServer::new(
        || App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(home_page)
            .service(handlerer)
            .service(register)
            .service(signin)
    ).bind("127.0.0.1:8000").expect("Could not bind to 8000 port").run();

    // rocket::ignite().mount("/",
    // routes![
    //     home_page,
    //     handlerer,
    //     signin,
    //     register,
    //     verify_session,
    //     verify_email,
    //     test_post
    // ]).launch();
}
