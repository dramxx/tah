use crate::messages::Message;
use chrono::{Local, TimeZone};
use colored::Colorize;

const HOUR_SECONDS: i64 = 3600;

pub struct ChatMessage {
    pub sender: String,
    pub message: Message,
}

pub fn render_messages(messages: &[ChatMessage], identity: &str, unread_marker_ts: Option<i64>) {
    let mut last_ts: Option<i64> = None;
    let now = Local::now();
    let mut unread_divider_printed = false;

    for cm in messages {
        if let Some(prev_ts) = last_ts {
            if cm.message.ts - prev_ts > HOUR_SECONDS {
                println!();
            }
        }

        let is_self = cm.sender == identity;
        let is_unread = !is_self && unread_marker_ts.is_some_and(|ts| cm.message.ts > ts);

        if is_unread && !unread_divider_printed {
            if last_ts.is_some() {
                println!();
            }
            println!("{}", "--- unread ---".bold().blue());
            unread_divider_printed = true;
        }

        let local_time = Local.timestamp_opt(cm.message.ts, 0)
            .single()
            .unwrap_or_else(|| now.clone());

        let timestamp_str = if local_time.date_naive() == now.date_naive() {
            local_time.format("%H:%M").to_string()
        } else {
            local_time.format("%a %H:%M").to_string()
        };

        let line = format!("[{}] {}: {}", timestamp_str, cm.sender, cm.message.text);
        if is_self {
            println!("{}", line.bold().cyan());
        } else {
            println!("{}", line.yellow());
        }

        last_ts = Some(cm.message.ts);
    }
}
