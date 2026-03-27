# tah

A minimal CLI chat tool that uses GitHub Gist as a backend for 1:1 messaging.

## What is this?

`tah` lets you chat with someone using a shared GitHub Gist. Each person writes to their own `.jsonl` file in the gist, and messages are merged and displayed in chronological order. Pull-based — no polling, no background process.

Unread received messages are shown with an `--- unread ---` divider, `tah` emits a terminal bell when unread messages are present in the rendered view, and read state is tracked in `~/.config/tah/last_read` on Linux/macOS or `%APPDATA%\tah\last_read` on Windows.

## Storage

Config lives at `~/.config/tah/config.toml` on Linux/macOS or `%APPDATA%\tah\config.toml` on Windows:

```toml
token = "ghp_xxx"
gist_id = "abc123"
identity = "dma"
peer = "marian"
```

## Install

```bash
cargo build --release
# Binary at target/release/tah
```

## Usage

```bash
tah --init              # First-time setup
tah --config            # Open config file in editor
tah "hello there"      # Send a message
tah                     # Show the latest messages
tah history             # Show all messages
tah history --sent      # Show only your messages
tah history --received  # Show only received messages
```

The default view shows the latest 20 messages, but expands automatically to include older unread received messages so they are not marked as read without being displayed.

## Known Limitations

- **No polling** — Messages are only fetched when you run `tah`
- **No push notifications** — Unread messages only trigger a terminal bell when you open the chat
- **No encryption** — Messages are stored in plain text in the gist
- **No group chat** — Only supports 1:1 conversations
- **No message editing or deletion**
