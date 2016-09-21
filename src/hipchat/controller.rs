use std::time::Duration;
use hyper;
use hyper::client::Client;
use super::{ChatMessage, ChatResponse, RoomItem, RoomResponse};
use super::{ApiRequest, ApiResponse};
use serde_json;
use rusqlite::{Connection, Transaction};
use db;
use chrono::{UTC, DateTime};

pub struct Controller {
    pub auth: String,
    pub server: String,
    pub hyper: Client,
}

impl Controller {
    pub fn new(auth: String, server: String) -> Controller {
        let mut client: Client = Client::new();
        client.set_read_timeout(Some(Duration::from_secs(30)));
        client.set_write_timeout(Some(Duration::from_secs(30)));
        Controller {
            auth: auth,
            server: server,
            hyper: client,
        }
    }

    pub fn get_rooms(&self) -> Vec<RoomItem> {
        let mut ret: Vec<RoomItem> = Vec::with_capacity(3000);
        let mut url = hyper::Url::parse(&format!("https://{}/v2/room", self.server)).unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("include-archived", "true")
            .append_pair("include-private", "false")
            .append_pair("max-results", "999")
            ;
        let mut room_address: String = url.as_str().to_owned();
        let mut req: ApiRequest = ApiRequest::new(room_address);
        loop {
            req = {
                let res: ApiResponse = req.send(&self);

                // Actually handle response here
                let decoded: RoomResponse = serde_json::from_str(res.get_content()).unwrap();
                ret.extend(decoded.items.iter().cloned());

                match res.get_next_request() {
                    None => {break;},
                    Some(new_request) => {
                        println!("Advancing to {}", new_request.get_url());
                        new_request
                    }
                }
            }
        }
        ret
    }

    pub fn get_messages_for_room(&self, id: i32) {
        let mut conn: Connection = db::open_db();
        let mut url = hyper::Url::parse(&format!("https://{}/v2/room/{}/history", self.server, id)).unwrap();
        let now_string: String = time_to_8061(UTC::now());
        let date_in_room: i64 = db::get_most_recent_timestamp_for_room(&conn, id);

        url.query_pairs_mut()
            .clear()
            .append_pair("reverse", "false")
            .append_pair("timezone", "UTC")
            .append_pair("date", &now_string)
            .append_pair("max-results", if date_in_room > 0 { "100" } else { "999"} ) // For some reason, 1000 breaks paging
            ;

        let mut room_address: String = url.as_str().to_owned();
        let mut req: ApiRequest = ApiRequest::new(room_address);
        let mut db_already_has_message: bool = false;
        let mut num_added: u64 = 0;
        
        // Hipchat starts listing messages from the most recent with
        // paging going backwards so we need to do a transaction to
        // make sure we don't get a partial insert since we will stop
        // paging once we hit a message already in the DB
        let tx: Transaction = conn.transaction().unwrap();

        while !db_already_has_message {
            req = {
                let res: ApiResponse = req.send(&self);

                // Actually handle the response here
                let decoded: ChatResponse = match serde_json::from_str(res.get_content()) {
                    Ok(dec) => dec,
                    Err(e) => {
                        panic!("Error decoding json. Error: {:?}.\n{}", e, res.get_content());
                    }
                };
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

                // Paging
                match res.get_next_request() {
                    None => {break;},
                    Some(new_request) => {
                        println!("Advancing to {}", new_request.get_url());
                        new_request
                    }
                }
            }
        }
        // At this point the entire history for the room should be in the
        // DB so we can commit the transaction
        tx.commit();
    }
}

fn time_to_8061(time: DateTime<UTC>) -> String {
    time.format("%Y-%m-%dT%H:%M:%S+00:00").to_string().to_owned()
}
