use hyper;
use hyper::client::Client;
use hyper::header::Authorization;
use std::io::Read;
use std::io::Write;
use std::collections::BTreeMap;
use rustc_serialize::json::{self, Json, ToJson};

use db;

#[derive(RustcDecodable)]
pub struct RoomResponse {
    items: Vec<RoomItem>,
    links: BTreeMap<String, String>,
    maxResults: u32,
    startIndex: u32,
}

#[derive(RustcDecodable)]
pub struct RoomItem {
    id: u32,
    is_archived: bool,
    links: BTreeMap<String, String>,
    name: String,
    privacy: String,
    version: String,
}

pub fn get_rooms() {
    let api_key: String = db::get_db_property("api_key").expect("DB Missing api_key");
    let server:  String = db::get_db_property("server").expect("DB Missing server");
    let auth = format!("Bearer {}", api_key);
    let client = Client::new();

    let mut room_address: String = format!("https://{}/v2/room", server);
    
    loop {
        let mut res = client.get(&room_address)
            .header(Authorization(auth.clone()))
            .send().unwrap();
        assert_eq!(res.status, hyper::Ok);
        let mut content = String::new();
        let size_read = res.read_to_string(&mut content);
        let decoded: RoomResponse = json::decode(&content).unwrap();
        let next: Option<&String> = decoded.links.get("next");
        if next.is_none() {
            break;
        } else {
            room_address = next.unwrap().to_owned();
            println!("Advancing to {}", room_address);
        }
    }
}
