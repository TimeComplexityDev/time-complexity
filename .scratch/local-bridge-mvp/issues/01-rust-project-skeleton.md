# 01 — Rust project skeleton and CLI

**What to build:** A runnable `apps/local-bridge` Rust binary whose CLI parses the flags needed to launch the bridge.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `cargo run --` produces a working binary with `--help`
- [ ] `--port <u16>` accepted (default 9001)
- [ ] `--pair-token <str>` accepted; if omitted, generate a random token and print it
- [ ] `--input-file <path>` accepted for offline MP3/WAV testing
- [ ] `--reset-pairing` accepted to revoke existing token
- [ ] Token persisted to a config file under `~/.config/timebridge/` or repo-local `.timebridge/`
- [ ] Binary builds with no warnings on stable Rust
