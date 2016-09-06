use hyper;
use hyper::client::Client;
use hyper::header::Authorization;
use hyper::client::response::Response;
use std::io::Read;
use std::collections::BTreeMap;
use time;
use serde_json;
use std::thread;
use std::time::Duration;
use rusqlite::{Connection, Transaction};

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
    pub color: Option<String>,
    pub date: String,
    pub from: serde_json::Value,
    pub id: String,
    pub message: String,
    pub message_format: Option<String>,
}

pub fn get_rooms() -> Vec<RoomItem> {
    let conn: Connection = db::open_db();
    let mut ret: Vec<RoomItem> = Vec::with_capacity(3000);
    let api_key: String = db::get_db_property(&conn, "api_key").expect("DB Missing api_key");
    let server:  String = db::get_db_property(&conn, "server").expect("DB Missing server");
    let auth = format!("Bearer {}", api_key);
    let client = Client::new();

    let mut url = hyper::Url::parse(&format!("https://{}/v2/room", server)).unwrap();
    url.query_pairs_mut()
        .clear()
        .append_pair("include-archived", "true")
        .append_pair("include-private", "false")
        .append_pair("max-results", "999")
        ;
    
    let mut room_address: String = url.as_str().to_owned();
    
    loop {
        let mut res = client.get(&room_address)
            .header(Authorization(auth.clone()))
            .send().unwrap();
        if maybe_rate_limited(&res) {
            continue;
        }
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

fn unix_to_8061(seconds: i64) -> String {
    let now = time::Timespec {
        sec: seconds,
        nsec: 0
    };
    let now_time = time::at_utc(now);
    let without_timezone: String = time::strftime("%Y-%m-%dT%H:%M:%S", &now_time).unwrap();
    format!("{}+00:00", without_timezone)
}

fn maybe_rate_limited(res: &Response) -> bool {
    match res.status {
        hyper::Ok => false,
        hyper::status::StatusCode::TooManyRequests => {
            println!("Hitting rate limit, sleeping for 6 minutes");
            let five_minutes = Duration::from_secs(6 * 60);
            thread::sleep(five_minutes);
            true
        },
        _ => {
            panic!("Unknown status code {}", res.status);
        }
    }
        
}

pub fn get_messages_for_room(id: i32) {
    let mut conn: Connection = db::open_db();
    let api_key: String = db::get_db_property(&conn, "api_key").expect("DB Missing api_key");
    let server:  String = db::get_db_property(&conn, "server").expect("DB Missing server");
    let auth = format!("Bearer {}", api_key);
    let client = Client::new();

    let mut url = hyper::Url::parse(&format!("https://{}/v2/room/{}/history", server, id)).unwrap();
    let now = time::get_time();
    let now_string: String = unix_to_8061(now.sec);
    
    url.query_pairs_mut()
        .clear()
        .append_pair("reverse", "false")
        .append_pair("timezone", "UTC")
        .append_pair("date", &now_string)
        .append_pair("max-results", "999") // For some reason, 1000 breaks paging
        ;
    
    let mut room_address: String = url.as_str().to_owned();
    let mut db_already_has_message: bool = false;
    let mut num_added: u64 = 0;
    // Hipchat starts listing messages from the most recent with
    // paging going backwards so we need to do a transaction to make
    // sure we don't get a partial insert since we will stop paging
    // once we hit a message already in the DB
    let tx: Transaction = conn.transaction().unwrap();

    while !db_already_has_message {
        let mut res = client.get(&room_address)
            .header(Authorization(auth.clone()))
            .send().unwrap();
        if maybe_rate_limited(&res) {
            continue;
        }
        assert_eq!(res.status, hyper::Ok);
        let mut content = String::new();
        let size_read = res.read_to_string(&mut content);

        let decoded: ChatResponse = serde_json::from_str(&content).unwrap();

        for msg in decoded.items {
            if db::add_message(&tx, &msg, id) {
                if ! db_already_has_message {
                    println!("DB already has message!");
                }
                db_already_has_message = true;
            } else {
                num_added += 1;
            }
        }

        println!("Added so far: {}", num_added);
        let next: Option<&String> = decoded.links.get("next");
        if next.is_none() {
            break;
        } else {
            room_address = next.unwrap().to_owned();
            println!("Advancing to {}", room_address);
        }
    }
    // At this point the entire history for the room should be in the
    // DB so we can commit the transaction
    tx.commit();
}
