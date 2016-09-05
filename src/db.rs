use rusqlite::SqliteConnection;
use std::path::Path;

#[derive(Debug, Clone)]
struct DbProperty {
    name: String,
    value: Option<String>,
}

pub fn get_db_property(name: &str) -> Option<String>
{
    let conn = SqliteConnection::open(Path::new("data.db")).unwrap();
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
