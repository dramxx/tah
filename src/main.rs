mod config;
mod display;
mod gist;
mod messages;

use clap::{Parser, Subcommand};
 use config::Config;
use display::{render_messages, ChatMessage};
use gist::GistClient;
use messages::Message;
 use std::collections::HashSet;
 use std::io::{self, Write};

const DEFAULT_MESSAGE_LIMIT: usize = 20;

#[derive(Parser)]
#[command(name = "tah")]
#[command(about = "Chat with someone via GitHub Gist")]
struct Cli {
    #[arg(long)]
    init: bool,
    #[arg(long)]
    config: bool,
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(last = true)]
    text: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    History {
        #[arg(long, conflicts_with = "received")]
        sent: bool,
        #[arg(long, conflicts_with = "sent")]
        received: bool,
    },
}

#[derive(Clone, Copy)]
enum MessageFilter {
    All,
    Sent,
    Received,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.init {
        if cli.config || cli.command.is_some() || !cli.text.is_empty() {
            return Err("error: '--init' cannot be combined with other arguments".into());
        }

        do_init()?;
        return Ok(());
    }

    if cli.config {
        if cli.command.is_some() || !cli.text.is_empty() {
            return Err("error: '--config' cannot be combined with other arguments".into());
        }

        do_config()?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::History { sent, received }) => {
            if !cli.text.is_empty() {
                return Err("error: 'history' does not accept a message".into());
            }

            let config = Config::load()?;
            do_history(&config, sent, received)?;
        }
        None => {
            let config = Config::load()?;

            if cli.text.is_empty() {
                do_show(&config, false)?;
            } else {
                let text = cli.text.join(" ");
                do_send(&config, &text)?;
            }
        }
    }

    Ok(())
}

fn fetch_messages(config: &Config) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
    let client = GistClient::new(config);

    let identity_file = format!("{}.jsonl", config.identity);
    let peer_file = format!("{}.jsonl", config.peer);

    let gist = client.get_gist()?;

    let identity_content = client.file_content(&gist, &identity_file)?;
    let peer_content = client.file_content(&gist, &peer_file)?;

    let identity_msgs = messages::parse_jsonl(&identity_content);
    let peer_msgs = messages::parse_jsonl(&peer_content);

    let mut chat_messages: Vec<ChatMessage> = Vec::with_capacity(identity_msgs.len() + peer_msgs.len());
    for m in identity_msgs {
        chat_messages.push(ChatMessage {
            sender: config.identity.clone(),
            message: m,
        });
    }
    for m in peer_msgs {
        chat_messages.push(ChatMessage {
            sender: config.peer.clone(),
            message: m,
        });
    }

    let mut seen_ids = HashSet::new();
    chat_messages.retain(|cm| seen_ids.insert(cm.message.id.clone()));
    chat_messages.sort_by(|left, right| {
        left.message
            .ts
            .cmp(&right.message.ts)
            .then_with(|| left.message.id.cmp(&right.message.id))
    });

    Ok(chat_messages)
}

fn do_init() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting interactive setup...");
    let config = Config::interactive_init()?;

    let client = GistClient::new(&config);
    println!("Testing token and gist...");
    let gist = client.get_gist()?;

    let filename = format!("{}.jsonl", config.identity);
    if !gist.files.contains_key(&filename) {
        println!("Creating your message file...");
        client.update_file(&filename, "")?;
    }

    config.save()?;
    println!("✓ Setup complete. Run 'tah' to see chat history.");
    Ok(())
}

fn do_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = Config::existing_config_path()
        .ok_or("error: not initialized. Run 'tah --init'")?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("notepad")
            .arg(&path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()?;
    }
    
    Ok(())
}

fn do_show(config: &Config, full: bool) -> Result<(), Box<dyn std::error::Error>> {
    do_render(config, full, MessageFilter::All)
}

fn do_history(config: &Config, sent: bool, received: bool) -> Result<(), Box<dyn std::error::Error>> {
    let filter = if sent {
        MessageFilter::Sent
    } else if received {
        MessageFilter::Received
    } else {
        MessageFilter::All
    };

    do_render(config, true, filter)
}

fn do_render(
    config: &Config,
    full: bool,
    filter: MessageFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut chat_messages = fetch_messages(config)?;
    let last_read = Config::load_last_read();

    match filter {
        MessageFilter::All => {}
        MessageFilter::Sent => chat_messages.retain(|cm| cm.sender == config.identity),
        MessageFilter::Received => chat_messages.retain(|cm| cm.sender == config.peer),
    }

    if !full {
        let default_start = chat_messages.len().saturating_sub(DEFAULT_MESSAGE_LIMIT);
        let unread_start = chat_messages.iter().position(|cm| {
            cm.sender == config.peer && cm.message.ts > last_read
        });
        let start = unread_start.map_or(default_start, |index| index.min(default_start));

        chat_messages = chat_messages.into_iter().skip(start).collect();
    }

    let marks_read = !matches!(filter, MessageFilter::Sent);
    let latest_displayed_peer_ts = chat_messages
        .iter()
        .filter(|cm| cm.sender == config.peer)
        .map(|cm| cm.message.ts)
        .max();
    let has_unread = marks_read && latest_displayed_peer_ts.is_some_and(|ts| ts > last_read);

    render_messages(
        &chat_messages,
        &config.identity,
        has_unread.then_some(last_read),
    );

    if has_unread {
        print!("\u{7}");
        io::stdout().flush()?;
    }

    if marks_read {
        mark_messages_read(latest_displayed_peer_ts, last_read)?;
    }

    Ok(())
}

fn mark_messages_read(
    latest_displayed_peer_ts: Option<i64>,
    last_read: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(timestamp) = latest_displayed_peer_ts.filter(|ts| *ts > last_read) {
        Config::save_last_read(timestamp)?;
    }

    Ok(())
}

fn do_send(config: &Config, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = text.trim();
    if text.is_empty() {
        return Err("error: message cannot be empty".into());
    }

    let client = GistClient::new(config);
    let filename = format!("{}.jsonl", config.identity);

    let new_msg = Message::new(text.to_string());
    let new_line = serde_json::to_string(&new_msg).map_err(|e| format!("error: {}", e))?;

    client.append_to_file(&filename, &new_line)?;

    println!("✓ sent");
    Ok(())
}
