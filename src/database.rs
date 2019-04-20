use redis::Commands;
use redis::Connection;
use std::sync::Mutex;

use super::SETTINGS;

pub fn connect_to_redis() -> Mutex<Connection> { //use settings
// @todo add AUTH
    let client = redis::Client::open("redis://127.0.0.1/").expect("Could not open connection");
    Mutex::new(client.get_connection().expect("Could not get connection"))
}