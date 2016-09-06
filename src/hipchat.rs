use hyper;
use hyper::client::Client;
use hyper::header::Authorization;
use std::io::Read;
use std::collections::BTreeMap;
use time;
use serde_json;

use db;

#[derive(Deserialize)]
pub struct RoomResponse {
    items: Vec<RoomItem>,
    links: BTreeMap<String, String>,
    maxResults: u32,
    startIndex: u32,
}

#[derive(Deserialize, Clone)]
pub struct RoomItem {
    pub id: i32,
    pub is_archived: bool,
    links: BTreeMap<String, String>,
    pub name: String,
    pub privacy: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    items: Vec<ChatMessage>,
    links: BTreeMap<String, String>,
    maxResults: u32,
    startIndex: u32,
}

#[derive(Deserialize)]
pub struct ChatMessage {
    color: Option<String>,
    date: String,
    from: serde_json::Value,
    id: String,
    message: String,
    message_format: Option<String>,
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
        let decoded: RoomResponse = serde_json::from_str(&content).unwrap();

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

pub fn get_messages_for_room(id: i32) {
    let api_key: String = db::get_db_property("api_key").expect("DB Missing api_key");
    let server:  String = db::get_db_property("server").expect("DB Missing server");
    let auth = format!("Bearer {}", api_key);
    let client = Client::new();

    let mut url = hyper::Url::parse(&format!("https://{}/v2/room/{}/history", server, id)).unwrap();
    let now = time::get_time();

    url.query_pairs_mut()
        .clear()
        .append_pair("reverse", "true")
        .append_pair("timezone", "UTC")
        .append_pair("date", &now.sec.to_string())
        ;
    
    let mut room_address: String = url.as_str().to_owned();
    loop {
        let mut res = client.get(&room_address)
            .header(Authorization(auth.clone()))
            .send().unwrap();
        assert_eq!(res.status, hyper::Ok);
        let mut content = String::new();
        let size_read = res.read_to_string(&mut content);

        let decoded: ChatResponse = serde_json::from_str(&content).unwrap();

        let next: Option<&String> = decoded.links.get("next");
        if next.is_none() {
            break;
        } else {
            room_address = next.unwrap().to_owned();
            println!("Advancing to {}", room_address);
        }
        break;
    }
}
