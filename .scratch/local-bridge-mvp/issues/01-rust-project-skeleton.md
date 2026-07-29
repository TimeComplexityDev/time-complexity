# 01 — Rust project skeleton and CLI

**What to build:** A runnable `apps/local-bridge` Rust binary whose CLI parses the flags needed to launch the bridge.

**Blocked by:** None — can start immediately.

**Labels:** ready-for-agent

**Status:** done

- [x] `cargo run --` produces a working binary with `--help`
- [x] `--port <u16>` accepted (default 9001)
- [x] `--pair-token <str>` accepted; if omitted, generate a random token and print it
- [x] `--input-file <path>` accepted for offline MP3/WAV testing
- [x] `--reset-pairing` accepted to revoke existing token
- [x] Token persisted to a config file under `~/.config/timebridge/`
- [x] Binary builds with no warnings on stable Rust
