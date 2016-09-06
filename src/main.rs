#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate rusqlite;
extern crate time;
extern crate serde;
extern crate serde_json;

pub mod db;
pub mod hipchat;

use hipchat::RoomItem;

fn main() {
    {
        let conn = db::open_db();
        let rooms: Vec<RoomItem> = hipchat::get_rooms();
        db::update_rooms(&conn, rooms);
    }
    {
        let conn = db::open_db();
        for id in db::get_all_room_ids(&conn) {
            println!("Starting room {}", id);
            hipchat::get_messages_for_room(id);
        }
    }
    println!("Finished!");
}
