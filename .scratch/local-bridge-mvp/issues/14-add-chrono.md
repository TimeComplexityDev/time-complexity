# 14 — Refactor: replace hand-rolled date arithmetic with chrono

**What to build:** Replace the 50-line hand-rolled `chrono_now_iso` function in `metrics.rs` with the well-established `chrono` crate. This eliminates a maintenance burden and gives correct timezone handling.

**Blocked by:** None

**Labels:** ready-for-agent

**Status:** done

## Changes

- Add `chrono = { version = "0.4", features = ["clock"] }` to `Cargo.toml`
- Replace `chrono_now_iso` in `metrics.rs` with:
  ```rust
  fn chrono_now_iso() -> String {
      chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
  }
  ```
- Remove `is_leap` helper and month-day arrays (no longer needed)
- Delete the `chrono_now_iso` function body (~50 lines) entirely

## Rationale

The hand-rolled version was speculative generality to avoid a dependency. `chrono` is the standard Rust date-time library — no reason to avoid it now. The `time` field on aggregate messages is a spec requirement, not nice-to-have.

**File:** `apps/local-bridge/Cargo.toml`, `apps/local-bridge/src/metrics.rs`