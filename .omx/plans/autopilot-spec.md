# clawhip autopilot spec

## Product
clawhip is a standalone Rust daemon/CLI that routes inbound events to Discord channels using a dedicated bot token and the Discord REST API. It is independent of clawdbot plugins and gateways.

## Required capabilities
1. CLI `custom` command sends arbitrary text notifications.
2. CLI `github issue-opened` command emits structured GitHub issue-opened notifications.
3. CLI `tmux keyword` command emits structured tmux keyword notifications.
4. CLI `stdin` ingests JSON/NDJSON events.
5. CLI `serve --port 8765` exposes webhook endpoints for generic events and GitHub issues.
6. CLI `config` provides an interactive editor for config state.
7. Config stored at `~/.clawhip/config.toml` with Discord token, defaults, routes, and optional templates.
8. Discord message delivery uses reqwest against Discord REST API.
9. Per-route formatting supports `compact`, `alert`, `inline`, and `raw`.

## Event model
- Canonical event names: `custom`, `github.issue-opened`, `tmux.keyword`
- Input envelope fields: `type`, optional `channel`, optional `format`, optional `template`, `payload`
- Route resolution precedence: explicit event channel -> route channel -> default channel
- Format precedence: explicit event format -> route format -> default format
- Template precedence: explicit event template -> route template -> built-in renderer

## Operational constraints
- Missing bot token is a hard error for send/serve paths.
- End-to-end validation must prove `custom` sends a Discord-compatible POST request.
