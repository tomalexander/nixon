use rusqlite::{Connection, SqliteConnection};
use std::path::Path;

use hipchat::RoomItem;

#[derive(Debug, Clone)]
struct DbProperty {
    name: String,
    value: Option<String>,
}

fn open_db() -> Connection {
    SqliteConnection::open(Path::new("data.db")).unwrap()
}

pub fn get_db_property(name: &str) -> Option<String>
{
    let conn = open_db();
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

pub fn update_rooms(rooms: Vec<RoomItem>) {
    let conn = open_db();

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
