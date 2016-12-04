#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

#[macro_use] extern crate hyper;
extern crate rusqlite;
extern crate chrono;
extern crate serde;
extern crate serde_json;

pub mod db;
pub mod hipchat;

fn main() {
    let controller: hipchat::Controller = {
        let conn: rusqlite::Connection = db::open_db();
        let api_key: String = db::get_db_property(&conn, "api_key").expect("DB Missing api_key");
        let server:  String = db::get_db_property(&conn, "server").expect("DB Missing server");
        let auth = format!("Bearer {}", api_key);
        hipchat::Controller::new(auth, server)
    };
    {
        let conn: rusqlite::Connection = db::open_db();
        let rooms: Vec<hipchat::RoomItem> = controller.get_rooms();
        db::update_rooms(&conn, rooms);
    }
    {
        let conn = db::open_db();
        for id in db::get_all_room_ids(&conn) {
            println!("Starting room {}", id);
            controller.get_messages_for_room(id);
        }
    }
    println!("Finished!");
}
