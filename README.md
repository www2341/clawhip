<p align="center">
  <img src="assets/clawhip-mascot.jpg" alt="clawhip mascot" width="500">
</p>

<h1 align="center">🦞🔥 clawhip</h1>

<p align="center">
  <strong>claw + whip</strong> — Event-to-channel notification router<br>
  <em>The daemon that whips your clawdbot into shape.</em>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#install">Install</a> •
  <a href="#usage">Usage</a> •
  <a href="#config">Config</a>
</p>

---

## What is clawhip?

**clawhip** is a standalone notification router that takes events (GitHub webhooks, tmux pane keywords, custom messages, cron triggers) and routes them directly to Discord channels — bypassing gateway sessions entirely to avoid context pollution.

It's an independent bot. No clawdbot plugin, no gateway integration. Just a fast Rust binary with its own Discord bot token.

> *Your claw. Your whip. Your daemon, your rules.*

## Features

- 🔔 **Event routing** — GitHub, tmux, custom, stdin, HTTP webhooks
- 💬 **Discord delivery** — Direct REST API, no gateway needed
- ⚙️ **CLI-first** — Configure everything from the command line
- 📋 **Flexible formats** — compact, alert, inline, raw per-route
- 🚀 **Fast** — Single Rust binary, minimal footprint
- 🌐 **Webhook server** — `clawhip serve` for GitHub/external webhooks

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Send a custom notification
clawhip custom --channel 1468539002985644084 --message "Build complete! 🟢"

# GitHub event
clawhip github issue-opened --repo oh-my-claudecode --number 1460 --title "Bug in setup"

# tmux keyword detection
clawhip tmux keyword --session issue-1440 --keyword "PR created" --line "PR #1453 merged"

# Pipe JSON events
echo '{"type":"custom","channel":"1468539002985644084","message":"Hello!"}' | clawhip stdin

# Start webhook server
clawhip serve --port 8765

# Manage config
clawhip config show
clawhip config set token <BOT_TOKEN>
clawhip config set default-channel <CHANNEL_ID>
```

## Config

Config lives at `~/.clawhip/config.toml`:

```toml
[bot]
token = "your-discord-bot-token"
default_channel = "1468539002985644084"

[[routes]]
event = "github.ci-failed"
channel = "1468539002985644084"
format = "alert"

[[routes]]
event = "tmux.error"
channel = "1468539002985644084"
format = "alert"

[[routes]]
event = "github.*"
channel = "1468539002985644084"
format = "compact"
```

### Message Formats

| Format | Use Case | Example |
|--------|----------|---------|
| `compact` | Routine updates | **[PR merged]** fix: dedupe notifications \| url |
| `alert` | Failures/urgent | 🚨 **CI Failed** — oh-my-claudecode#1453 |
| `inline` | tmux events | `issue-1440` → PR created |
| `raw` | Custom messages | Whatever you send |

## Architecture

```
[Event Source] → clawhip CLI/HTTP → [Route Engine] → [Discord REST API] → [Channel]
```

No gateway. No session pollution. Just events in, messages out.

## License

MIT
