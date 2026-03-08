<p align="center">
  <img src="assets/clawhip-mascot.jpg" alt="clawhip mascot" width="500">
</p>

<h1 align="center">🦞🔥 clawhip</h1>

<p align="center">
  <strong>claw + whip</strong> — standalone event gateway to Discord<br>
  <em>All events come in through CLI or webhooks, then route to channels.</em>
</p>

---

## Core architecture

`clawhip` is a **gateway**, not a plugin and not a Discord gateway client.

The model is:

```text
[external hook / cron / wrapper / webhook] -> [clawhip CLI or HTTP] -> [route matching] -> [Discord REST API]
```

That means:

- no clawdbot plugin integration
- no internal git polling daemon
- no internal tmux watch daemon
- git and tmux integrations are provided as **ready-to-use scripts** under `integrations/`
- `clawhip tmux new ... -- command` is a built-in wrapper for launching a tmux session with monitoring

## Features

- `clawhip custom --channel <id> --message <text>`
- `clawhip git commit ...`
- `clawhip git branch-changed ...`
- `clawhip github issue-opened ...`
- `clawhip github pr-status-changed ...`
- `clawhip tmux keyword ...`
- `clawhip tmux stale ...`
- `clawhip tmux new -s session --channel ... --keywords ... --stale-minutes ... -- command args`
- `clawhip stdin`
- `clawhip serve --port 8765`
- `clawhip config`
- route filters with payload-field matching and glob support
- Discord delivery via `reqwest`

## Install

```bash
cargo install --path .
```

## Basic usage

```bash
# Custom notification
clawhip custom --channel 1468539002985644084 --message "Build complete"

# Git commit event
clawhip git commit \
  --repo clawhip \
  --branch main \
  --commit deadbeefcafebabe \
  --summary "Ship gateway prototype"

# GitHub issue-opened event
clawhip github issue-opened \
  --repo clawhip \
  --number 42 \
  --title "Webhook regression"

# Pull-request status event
clawhip github pr-status-changed \
  --repo clawhip \
  --number 77 \
  --title "Add tmux wrapper" \
  --old-status open \
  --new-status merged \
  --url https://github.com/bellman/clawhip/pull/77

# tmux keyword event
clawhip tmux keyword \
  --session issue-1440 \
  --keyword "PR created" \
  --line "PR #1453 created"

# tmux stale event
clawhip tmux stale \
  --session issue-1440 \
  --pane 0.0 \
  --minutes 10 \
  --last-line "running integration tests"
```

## tmux wrapper mode

`clawhip tmux new` launches a tmux session and monitors its pane output for keywords and staleness.

```bash
clawhip tmux new -s issue-2000 \
  --channel 1468539002985644084 \
  --mention '<@botid>' \
  --keywords 'error,PR created,FAILED,complete' \
  --stale-minutes 10 \
  --format alert \
  -- cargo test
```

### Wrapper argument model

Arguments **before** `--` are clawhip/tmux wrapper options parsed by Clap:

- `-s, --session <name>`
- `--channel <id>`
- `--mention <tag>`
- `--keywords <comma-separated>`
- `--stale-minutes <n>`
- `--format <compact|alert|inline>`
- `-n, --window-name <name>`
- `-c, --cwd <dir>`
- `--attach`

Arguments **after** `--` are passed through as the command to run inside tmux.

## HTTP webhook gateway

```bash
clawhip serve --port 8765
```

Endpoints:

- `GET /health`
- `POST /events` — generic JSON events
- `POST /github` — GitHub `issues` and `pull_request` events

Supported GitHub webhook mappings:

- `issues.opened` -> `github.issue-opened`
- `pull_request.opened` -> `git.pr-status-changed` (`<new>` -> `open`)
- `pull_request.reopened` -> `git.pr-status-changed` (`closed` -> `open`)
- `pull_request.closed` -> `git.pr-status-changed` (`open` -> `closed` or `merged`)

## Config

Config lives at `~/.clawhip/config.toml`.

Example:

```toml
[discord]
bot_token = "your-discord-bot-token"

[defaults]
channel = "1468539002985644084"
format = "compact"

[[routes]]
event = "github.*"
filter = { repo = "oh-my-claudecode" }
channel = "1468539002985644084"
format = "compact"

[[routes]]
event = "github.*"
filter = { repo = "clawhip" }
channel = "9999999999"
format = "alert"

[[routes]]
event = "tmux.*"
filter = { session = "issue-*" }
channel = "1468539002985644084"
format = "compact"
```

### Route filtering

Routes are evaluated in config order. A route matches when:

1. `event` glob matches the event type, and
2. every `filter` entry matches the corresponding payload field

Filter values support glob patterns, so this works:

```toml
[[routes]]
event = "tmux.*"
filter = { session = "issue-*" }
channel = "1468539002985644084"
```

That lets the same event type route to different channels based on payload fields like `repo`, `session`, `branch`, etc.

### Environment overrides

- `CLAWHIP_CONFIG`
- `CLAWHIP_DISCORD_BOT_TOKEN`
- `CLAWHIP_DISCORD_API_BASE`
- `CLAWHIP_TMUX_BIN`
- `CLAWHIP_TMUX_POLL_SECS`

## JSON event gateway

`clawhip stdin` and `POST /events` accept flat or payload-style JSON.

Flat example:

```json
{
  "type": "custom",
  "channel": "1468539002985644084",
  "message": "Deploy completed"
}
```

Payload example:

```json
{
  "type": "git.commit",
  "payload": {
    "repo": "clawhip",
    "branch": "main",
    "commit": "deadbeefcafebabe",
    "summary": "Ship gateway prototype"
  }
}
```

## integrations/

Ready-to-use examples live in `integrations/`:

### Git

- `integrations/git/post-commit.sh`
- `integrations/git/post-checkout.sh`
- `integrations/git/install-hooks.sh`

Install example hooks into the current repo:

```bash
cd /path/to/repo
/path/to/clawhip/integrations/git/install-hooks.sh
```

Optional channel override:

```bash
export CLAWHIP_CHANNEL=1468539002985644084
```

### tmux

- `integrations/tmux/notify-keyword.sh`
- `integrations/tmux/scan-keywords.sh`
- `integrations/tmux/stale-check.sh`

Example cron entries:

```cron
* * * * * /path/to/clawhip/integrations/tmux/scan-keywords.sh --session issue-1440 --keywords error,FAILED,complete --channel 1468539002985644084
* * * * * /path/to/clawhip/integrations/tmux/stale-check.sh --session issue-1440 --stale-minutes 10 --channel 1468539002985644084
```

## Development

```bash
cargo fmt
cargo test
cargo build
```
