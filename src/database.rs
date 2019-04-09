use mongodb::coll::Collection;
use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use super::SETTINGS;

pub fn connect_to_database() -> Collection {
    let client = Client::connect(&SETTINGS.mongo.address.to_string(), SETTINGS.mongo.port).expect("Failed to connect to database");
    let db = client.db("admin");
    db.auth(&SETTINGS.mongo.user, &SETTINGS.mongo.password).expect("[Database] Failed to authenticate");
    client.db("thetahq").collection("users")
}