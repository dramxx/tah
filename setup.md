# tah — Setup Guide

You (dma) do all of this. Mayo just receives the token and gist ID from you at the end.

---

## Step 1: Create the Gist

1. Go to https://gist.github.com
2. Create a new **secret** gist (not public)
   - Filename: `dma.jsonl`
   - Content: leave it empty — `tah --init` will handle the rest
3. Click **Create secret gist**
4. Copy the Gist ID from the URL:
   ```
   https://gist.github.com/dramxx/abc123def456
                                    ^^^^^^^^^^^^^ this part
   ```

---

## Step 2: Create a GitHub PAT

1. Go to https://github.com/settings/tokens
2. Click **Generate new token → Generate new token (classic)**
3. Note / description: `tah-chat`
4. Expiration: set whatever you're comfortable with
5. Scopes: check only **`gist`**
6. Click **Generate token**
7. Copy the token immediately — GitHub won't show it again

> Fine-grained tokens don't support gist scope yet, so classic token is correct here.

---

## Step 3: Install tah (you)

```bash
git clone https://github.com/dramxx/tah
cd tah
cargo install --path .
```

Run setup:

```bash
tah --init
```

Enter when prompted:

- Token: the PAT from Step 2
- Gist ID: from Step 1
- Your username: `dma`
- Peer username: `mayo`

---

## Step 4: Share with Mayo

Send mayo two things:

```
gist id:  abc123def456
token:    ghp_xxxxxxxxxxxxxxxxxxxx
```

That's it. He doesn't need a GitHub account.

---

## Step 5: Mayo installs and sets up

Mayo runs:

```bash
git clone https://github.com/dramxx/tah
cd tah
cargo install --path .
tah --init
```

Mayo enters when prompted:

- Token: the one you sent him
- Gist ID: the one you sent him
- Your username: `mayo`
- Peer username: `dma`

`tah --init` will automatically create `mayo.jsonl` in the gist if it doesn't exist yet.

---

## Step 6: Verify it works

**You send:**

```bash
tah "yo mayo you there"
```

**Mayo checks:**

```bash
tah
```

Should see your message. If there are unread received messages in the rendered view, `tah` prints an `--- unread ---` divider and emits a terminal bell.

Mayo replies:

```bash
tah "hey dma, works"
```

You check:

```bash
tah
```

Done. You're chatting.

---

## Daily Usage

```bash
tah                        # see the latest messages
tah "text"                 # send a message
tah history                # full chat history
tah history --sent         # only your messages
tah history --received     # only received messages
tah --config               # open config.toml in your editor
```

---

## Notes

- Neither of you needs to be "online" — messages sit in the gist until the other checks
- No polling, no background process, nothing runs unless you invoke the command
- Read state is stored locally in `~/.config/tah/last_read` on Linux/macOS or `%APPDATA%\tah\last_read` on Windows
- If the token ever expires, generate a new one and both re-run `tah --init`
- The gist is secret but not encrypted — don't send anything you'd put in a public repo
