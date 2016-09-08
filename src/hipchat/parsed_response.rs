use serde_json;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct RoomResponse {
    pub items: Vec<RoomItem>,
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
    pub items: Vec<ChatMessage>,
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
    pub message: Option<String>,
    pub message_format: Option<String>,
}

