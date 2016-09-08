#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use] extern crate hyper;
extern crate rusqlite;
extern crate time;
extern crate serde;
extern crate serde_json;

pub mod db;
pub mod hipchat_old;
pub mod hipchat;

use hipchat_old::RoomItem;

fn main() {
    // {
    //     let conn = db::open_db();
    //     let rooms: Vec<RoomItem> = hipchat_old::get_rooms();
    //     db::update_rooms(&conn, rooms);
    // }
    // {
    //     let conn = db::open_db();
    //     for id in db::get_all_room_ids(&conn) {
    //         println!("Starting room {}", id);
    //         hipchat_old::get_messages_for_room(id);
    //     }
    // }
    {
        let conn: rusqlite::Connection = db::open_db();
        let api_key: String = db::get_db_property(&conn, "api_key").expect("DB Missing api_key");
        let server:  String = db::get_db_property(&conn, "server").expect("DB Missing server");
        let auth = format!("Bearer {}", api_key);
        let controller: hipchat::Controller = hipchat::Controller::new(auth);
        let mut url = hyper::Url::parse(&format!("https://{}/v2/room", server)).unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("include-archived", "true")
            .append_pair("include-private", "false")
            .append_pair("max-results", "999")
            ;
        let room_address: String = url.as_str().to_owned();
        let mut req: hipchat::ApiRequest = hipchat::ApiRequest::new(room_address);
        loop {
            req = {
                let res: hipchat::ApiResponse = req.send(&controller);
                match res.get_next_request() {
                    None => {break;},
                    Some(new_request) => {
                        println!("Advancing to {}", new_request.get_url());
                        new_request
                    }
                }
            }
        }
    }
    println!("Finished!");
}
