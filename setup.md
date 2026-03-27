# msg — Setup Guide

You (dma) do all of this. Mayo just receives the token and gist ID from you at the end.

---

## Step 1: Create the Gist

1. Go to https://gist.github.com
2. Create a new **secret** gist (not public)
   - Filename: `dma.jsonl`
   - Content: leave empty or put `{}` as placeholder — doesn't matter, init will handle it
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
3. Note / description: `msg-chat`
4. Expiration: set whatever you're comfortable with (1 year is fine for personal use)
5. Scopes: check only **`gist`**
6. Click **Generate token**
7. Copy the token immediately — GitHub won't show it again

> Fine-grained tokens don't support gist scope yet, so classic token is correct here.

---

## Step 3: Install msg (you)

```bash
git clone https://github.com/dramxx/msg
cd msg
cargo install --path .
```

Run setup:

```bash
msg --init
```

Enter when prompted:
- Token: the PAT from Step 2
- Gist ID: from Step 1
- Your username: `dma`
- Peer username: `mayo`

---

## Step 4: Share with Mayo

Send mayo (over Signal or whatever) two things:

```
gist id:  abc123def456
token:    ghp_xxxxxxxxxxxxxxxxxxxx
```

That's it. He doesn't need a GitHub account.

---

## Step 5: Mayo installs and sets up

Mayo runs:

```bash
git clone https://github.com/dramxx/msg
cd msg
cargo install --path .
msg --init
```

Mayo enters when prompted:
- Token: the one you sent him
- Gist ID: the one you sent him
- Your username: `mayo`
- Peer username: `dma`

`--init` will automatically create `mayo.jsonl` in the gist if it doesn't exist yet.

---

## Step 6: Verify it works

**You send:**
```bash
msg "yo mayo you there"
```

**Mayo checks:**
```bash
msg
```

Should see your message. Mayo replies:
```bash
msg "hey dma, works"
```

You check:
```bash
msg
```

Done. You're chatting.

---

## Daily Usage

```bash
msg                        # see last 20 messages
msg "text"                 # send a message
msg history                # full chat history
msg history --sent         # only your messages
msg history --received     # only mayo's messages
```

---

## Notes

- Neither of you needs to be "online" — messages sit in the gist until the other checks
- No polling, no background process, nothing running unless you invoke the command
- If the token ever expires, you generate a new one and both re-run `msg --init`
- The gist is secret but not encrypted — don't send anything you'd put in a public repo
