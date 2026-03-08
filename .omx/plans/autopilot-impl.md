# clawhip implementation plan

1. Replace skeleton crate with modular CLI/config/router/server implementation.
2. Implement config loading/saving from `~/.clawhip/config.toml`, with env overrides for testability.
3. Model event envelope + format rendering + lightweight templating.
4. Implement Discord REST client using reqwest and configurable API base URL.
5. Implement router that resolves channel/format/template and dispatches to Discord.
6. Implement `stdin` ingestion and `serve` HTTP endpoints (`/health`, `/events`, `/github`).
7. Add unit tests for route resolution and event parsing/rendering.
8. Run fmt/tests/build and end-to-end custom-command verification with a local mock Discord API.
9. Perform architecture/security/code review passes, then commit and push.
