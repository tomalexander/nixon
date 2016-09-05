extern crate hyper;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate time;

pub mod db;
pub mod hipchat;

use hipchat::RoomItem;

fn main() {
    // let rooms: Vec<RoomItem> = hipchat::get_rooms();
    // db::update_rooms(rooms);
    hipchat::get_messages_for_room(2);
}
