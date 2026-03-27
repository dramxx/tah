# msg — Implementation Plan

## Overview

A minimal Rust CLI for 1:1 chat between two people using GitHub Gist as a backend.
Pull-based, no polling, no background process, no notifications.

---

## Project Structure

```
msg/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point, command dispatch
│   ├── config.rs        # Load/save config from disk
│   ├── gist.rs          # GitHub Gist API client
│   ├── messages.rs      # Message struct, JSONL parse/serialize, merge logic
│   └── display.rs       # Terminal rendering (colored output)
```

---

## Config

Stored at:
- Linux/Mac: `~/.config/msg/config.toml`
- Windows: `%APPDATA%\msg\config.toml`

```toml
token = "ghp_xxxxxxxxxxxx"
gist_id = "abc123def456"
identity = "dma"          # your username, determines which file you write to
peer = "mayo"             # the other person's username
```

Loaded at startup. If missing, prompt user to run `msg --init`.

---

## Message Format

Each file in the gist is named `{identity}.jsonl` — one JSON object per line.

```json
{"id": "uuid-v4", "ts": 1712345678, "text": "hey whats up"}
{"id": "uuid-v4", "ts": 1712345999, "text": "you around?"}
```

Fields:
- `id` — UUIDv4, unique per message (useful for deduplication later if needed)
- `ts` — Unix timestamp seconds
- `text` — raw message string

No sender field needed — the file it lives in determines the sender.

---

## Gist Structure

Single gist with two files:

```
gist/
├── dma.jsonl
└── mayo.jsonl
```

Both users use the same token and gist ID. Each user only ever writes to their own file.

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
clap       = { version = "4", features = ["derive"] }
reqwest    = { version = "0.12", features = ["json", "blocking"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
toml       = "0.8"
uuid       = { version = "1", features = ["v4"] }
dirs       = "5"
chrono     = "0.4"
colored    = "2"
```

Use blocking reqwest to keep things simple — no async needed for a short-lived CLI.

---

## Commands

### `msg --init`

Interactive setup wizard:

1. Prompt: `Enter your GitHub PAT (gist scope):`
2. Prompt: `Enter the Gist ID:`
3. Prompt: `Enter your username (e.g. dma):`
4. Prompt: `Enter your peer's username (e.g. mayo):`
5. Test the token by hitting `GET /gists/{gist_id}` — validate it works
6. Check if `{identity}.jsonl` exists in gist — if not, create it empty via PATCH
7. Write config to disk
8. Print: `✓ Setup complete. Run 'msg' to see chat history.`

---

### `msg` (default, no args)

1. Load config
2. Fetch gist via `GET /gists/{gist_id}`
3. Extract content of `dma.jsonl` and `mayo.jsonl`
4. Parse both as JSONL into `Vec<Message>` with sender tagged
5. Merge and sort by `ts` ascending
6. Take last 20
7. Render (see Display section)

---

### `msg "text"`

1. Load config
2. Build new `Message { id, ts: now(), text }`
3. Fetch current content of `{identity}.jsonl` from gist
4. Append new message as JSON line
5. PATCH gist with updated file content
6. Print: `✓ sent`

---

### `msg history`

Same as `msg` but skip the `take last 20` — render everything.

---

### `msg history --sent`

Same as `msg history` but filter to only messages where sender == identity.

---

### `msg history --received`

Same as `msg history` but filter to only messages where sender == peer.

---

## Gist API

Base URL: `https://api.github.com`

**Auth header** (all requests):
```
Authorization: Bearer {token}
Accept: application/vnd.github+json
X-GitHub-Api-Version: 2022-11-28
User-Agent: msg-cli
```

**GET gist:**
```
GET /gists/{gist_id}
```
Response contains `files` map: `{ "dma.jsonl": { "content": "..." }, ... }`

**PATCH gist (update a file):**
```
PATCH /gists/{gist_id}
Content-Type: application/json

{
  "files": {
    "dma.jsonl": {
      "content": "<full updated file content as string>"
    }
  }
}
```

Note: PATCH replaces the entire file content. Always fetch first, append, then PATCH.

**Create file if it doesn't exist:**
Same PATCH endpoint — if the filename doesn't exist in the gist yet, GitHub creates it automatically.

---

## Display

Messages rendered in terminal, one per line:

```
[14:32] dma: hey whats up
[14:35] mayo: yo, here
[14:35] dma: nice, check this out
```

- Timestamp: local time, HH:MM format for today's messages; `Mon 14:32` for older ones
- Your messages: bold or colored (e.g. cyan)
- Peer messages: different color (e.g. yellow)
- Separator line printed if there's a gap of more than 1 hour between messages

---

## Error Handling

- Config missing → `error: not initialized. Run 'msg --init'`
- Token invalid (401) → `error: invalid token. Re-run 'msg --init'`
- Gist not found (404) → `error: gist not found. Check your gist ID.`
- Network error → `error: could not reach GitHub API: {reason}`
- Malformed JSONL line → skip silently (defensive parsing, don't crash)

---

## Edge Cases

- **Empty gist file:** Treat as zero messages, no error
- **Missing peer file:** Peer hasn't sent anything yet — treat as zero messages
- **Concurrent send (both at same time):** Each writes to their own file, no conflict possible
- **Clock skew:** Messages sorted by `ts` — if clocks differ slightly, order might look odd but won't break anything
- **Very long messages:** No truncation, render as-is and let the terminal wrap

---

## Out of Scope (for now)

- Notifications / polling
- Message deletion
- Editing sent messages
- Multiple peers / group chat
- Encryption
- Message size limits
