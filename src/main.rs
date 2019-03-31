#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};
use std::io;

#[get("/")]
fn home_page() -> io::Result<NamedFile> {
    NamedFile::open("static/build/index.html")
}

#[get("/<file..>", rank = 10)]
fn handlerer(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/build/").join(file)).ok().or_else(|| NamedFile::open(Path::new("static/build/index.html")).ok())
}

fn main() {
    rocket::ignite().mount("/", routes![home_page, handlerer]).launch();
}
