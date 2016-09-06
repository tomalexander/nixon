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
    // let rooms: Vec<RoomItem> = hipchat::get_rooms();
    // db::update_rooms(rooms);
    hipchat::get_messages_for_room(2);
}
