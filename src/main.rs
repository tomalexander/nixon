extern crate hyper;
extern crate rusqlite;
extern crate rustc_serialize;

pub mod db;
pub mod hipchat;

fn main() {
    hipchat::get_rooms();
}
