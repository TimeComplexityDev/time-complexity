# 12 — Refactor: split MetricsEngine::process_ticks into ingest + drain

**What to build:** `MetricsEngine::process_ticks` currently interleaves metric computation (rate, EWMA, beat error) with WebSocket message construction. Split into two phases so each has one reason to change.

**Blocked by:** None

**Labels:** ready-for-agent

**Status:** done

## New interface

```rust
impl MetricsEngine {
    /// Update internal state (EWMA, rates, beat error, half-periods) from a batch of ticks.
    /// Returns nothing — pure data transformation.
    pub fn ingest_ticks(&mut self, ticks: &[TickEvent]);

    /// Build WebSocket messages from the current state.
    /// Returns tick messages for all newly-ingested ticks and an aggregate if one second has elapsed.
    pub fn drain_messages(&mut self) -> (Vec<TickEventMessage>, Option<AggregateUpdate>);

    /// Read-only snapshot of the current metrics (for /status or other consumers).
    pub fn current_rate(&self) -> f64;
    pub fn current_beat_error(&self) -> f64;
    pub fn current_amplitude(&self) -> f64;
}
```

## Caller changes

- `handle_socket` in `main.rs` calls `ingest_ticks` then `drain_messages` every 50ms poll cycle.
- Future `/status` endpoint can call `current_*` methods directly instead of parsing from aggregate messages.

## Benefits

- Metric computation can be unit-tested without constructing JSON messages.
- Adding a new metric doesn't require touching serialization code.
- Other consumers (CLI, REST) can query the same engine without WebSocket coupling.

**Files:** `apps/local-bridge/src/metrics.rs`, `apps/local-bridge/src/main.rs`