extern crate hyper;
extern crate rusqlite;
extern crate rustc_serialize;

pub mod db;
pub mod hipchat;

use hipchat::RoomItem;

fn main() {
    let rooms: Vec<RoomItem> = hipchat::get_rooms();
    db::update_rooms(rooms);
}
