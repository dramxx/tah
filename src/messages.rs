use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub ts: i64,
    pub text: String,
}

impl Message {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().timestamp(),
            text,
        }
    }
}

pub fn parse_jsonl(content: &str) -> Vec<Message> {
    content
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            serde_json::from_str(line).ok()
        })
        .collect()
}
