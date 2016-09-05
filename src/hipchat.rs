use hyper;
use hyper::client::Client;
use hyper::header::Authorization;
use std::io::Read;
use std::collections::BTreeMap;
use rustc_serialize::json;

use db;

#[derive(RustcDecodable)]
pub struct RoomResponse {
    items: Vec<RoomItem>,
    links: BTreeMap<String, String>,
    maxResults: u32,
    startIndex: u32,
}

#[derive(RustcDecodable, Clone)]
pub struct RoomItem {
    pub id: i32,
    pub is_archived: bool,
    links: BTreeMap<String, String>,
    pub name: String,
    pub privacy: String,
    pub version: String,
}

pub fn get_rooms() -> Vec<RoomItem> {
    let mut ret: Vec<RoomItem> = Vec::with_capacity(3000);
    let api_key: String = db::get_db_property("api_key").expect("DB Missing api_key");
    let server:  String = db::get_db_property("server").expect("DB Missing server");
    let auth = format!("Bearer {}", api_key);
    let client = Client::new();

    let mut url = hyper::Url::parse(&format!("https://{}/v2/room", server)).unwrap();
    url.query_pairs_mut()
        .clear()
        .append_pair("include-archived", "true")
        .append_pair("include-private", "false");
    
    let mut room_address: String = url.as_str().to_owned();
    
    loop {
        let mut res = client.get(&room_address)
            .header(Authorization(auth.clone()))
            .send().unwrap();
        assert_eq!(res.status, hyper::Ok);
        let mut content = String::new();
        let size_read = res.read_to_string(&mut content);
        let decoded: RoomResponse = json::decode(&content).unwrap();

        ret.extend(decoded.items.iter().cloned());
        
        let next: Option<&String> = decoded.links.get("next");
        if next.is_none() {
            break;
        } else {
            room_address = next.unwrap().to_owned();
            println!("Advancing to {}", room_address);
        }
    }
    ret
}
