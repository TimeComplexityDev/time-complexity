# 11 — Refactor: consolidate BPH state to single source of truth

**What to build:** BPH currently lives in three locations (`device.bph`, `bph_override`, `pipeline.detected_bph`) that can diverge. Consolidate to two sources with a clear priority rule.

**Blocked by:** None — independent refactor

**Labels:** ready-for-agent

**Status:** done

## Model

- **`DspPipeline.detected_bph`** — set by the DSP on tick 10 from observed intervals. Single source of truth for detection.
- **`AppState.bph_override: Option<u32>`** — set by user via `/set_params` or WebSocket command. `None` when no override is active.
- **Effective BPH** = `bph_override.or(pipeline.detected_bph)` — never stored, computed on read.

## Changes

1. Remove `bph` field from `DeviceConfig` struct and `AppState.device` initialization.
2. In `status_handler`, compute effective BPH as `state.bph_override.or_else(|| state.pipeline.lock().ok().map(|p| p.detected_bph)).unwrap_or(28800)`.
3. In `set_params_handler`, stop writing to `device.bph` — only write to `bph_override`.
4. In `start_handler`, when creating session, do not reset `bph_override`.
5. In `handle_socket` WebSocket loop, remove the 50ms hammer that re-applies `bph_override` to `p.detected_bph` — only set `p.detected_bph` inside the pipeline when an override is first applied (not on every poll).
6. In `metrics::MetricsEngine`, accept the current BPH as a parameter to `process_ticks` or query it from the pipeline rather than storing it independently.

**File:** `apps/local-bridge/src/main.rs`, `apps/local-bridge/src/dsp.rs`, `apps/local-bridge/src/metrics.rs`