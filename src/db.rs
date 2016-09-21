use rusqlite::{Connection, SqliteConnection};
use std::path::Path;
use serde_json;
use chrono::DateTime;

use hipchat::{RoomItem, ChatMessage};

#[derive(Debug, Clone)]
struct DbProperty {
    name: String,
    value: Option<String>,
}

pub fn open_db() -> Connection {
    SqliteConnection::open(Path::new("data.db")).unwrap()
}

pub fn get_db_property(conn: &Connection, name: &str) -> Option<String>
{
    let mut stmt = conn.prepare("SELECT name, value FROM props WHERE name=$1").unwrap();
    let mut props = stmt.query_map(&[&name], |row| {
        DbProperty {
            name: row.get(0),
            value: row.get(1),
        }
    }).unwrap();

    match props.next() {
        Some(prop) => prop.unwrap().value,
        None => None
    }
}

pub fn update_rooms(conn: &Connection, rooms: Vec<RoomItem>) {
    let mut stmt = conn.prepare("INSERT OR REPLACE INTO rooms (id, is_archived, name, privacy, version) VALUES($1, $2, $3, $4, $5);").unwrap();
    for room in rooms {
        stmt.execute(&[&room.id,
                       &{if room.is_archived { 1 } else { 0 }},
                       &room.name,
                       &room.privacy,
                       &room.version,
        ]);
    }
}

fn has_message(id: &str, conn: &Connection) -> bool {
    let mut stmt = conn.prepare("SELECT id FROM messages WHERE id=$1;").unwrap();
    let mut messages = stmt.query_map(&[&id], |row| {
        let res: String = row.get(0);
        res
    }).unwrap();
    match messages.next() {
        Some(prop) => true,
        None => false
    }
}

fn message_date_to_unix(inp: &str) -> i64 {
    let tm = DateTime::parse_from_str(inp, "%+").unwrap(); // ISO 8061
    let ret: i64 = tm.timestamp() * 1000 + (tm.timestamp_subsec_millis() as i64);
    ret
}

fn message_from_to_string(inp: &serde_json::Value) -> String {
    match inp {
        &serde_json::Value::String(ref val) => val.clone(),
        &serde_json::Value::Object(ref val) => {
            val.get("name").map(|x| match x {
                &serde_json::Value::String(ref innerval) => innerval.clone(),
                _ => panic!("Name is not a string")
            }).unwrap_or("".to_owned())
        },
        _ => panic!("From is not a string or object")
    }
}

pub fn add_message(conn: &Connection, msg: &ChatMessage, room_id: i32) -> bool {
    let already_has_message: bool = has_message(&msg.id, &conn);
    
    if !already_has_message {
        let mut stmt = conn.prepare("INSERT INTO messages (room_id, id, color, date, sender, message, message_format) VALUES ($1, $2, $3, $4, $5, $6, $7);").unwrap();
        stmt.execute(&[&room_id,
                       &msg.id,
                       &msg.color,
                       &message_date_to_unix(&msg.date),
                       &message_from_to_string(&msg.from),
                       &msg.message,
                       &msg.message_format,
        ]);
    }
    already_has_message
}

pub fn get_most_recent_timestamp_for_room(conn: &Connection, room_id: i32) -> i64 {
    let mut stmt = conn.prepare("SELECT coalesce(max(date), 0) FROM messages WHERE room_id=$1;").unwrap();
    let mut messages = stmt.query_map(&[&room_id], |row| {
        let res: i64 = row.get(0);
        res
    }).unwrap();

    match messages.next() {
        Some(res_unix_timestamp) => match res_unix_timestamp {
            Ok(unix_timestamp) => unix_timestamp,
            Err(e) => 0,
        },
        None => 0
    }
}

pub fn get_all_room_ids(conn: &Connection) -> Vec<i32> {
    let mut stmt = conn.prepare("SELECT id FROM rooms;").unwrap();
    let mut ids = stmt.query_map(&[], |row| {
        let res: i32 = row.get(0);
        res
    }).unwrap();

    let ret: Vec<i32> = ids.map(|id| id.unwrap()).collect();
    ret
}
