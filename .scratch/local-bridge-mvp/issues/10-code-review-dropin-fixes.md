# 10 — Code review: batch drop-in fixes

**What to build:** Apply the quick, parallel-safe fixes identified by the holistic code review. Each item is self-contained and touches a small surface area.

**Blocked by:** None (independent of any pending ticket)

**Labels:** ready-for-agent

**Status:** done

## Fixes

### A1 — Document SafeStream unsafety

Add a comment explaining WHY `unsafe impl Send for SafeStream` and `unsafe impl Sync for SafeStream` are sound on macOS/CoreAudio. The wrapper exists because cpal 0.15 marks `Stream` as !Send/!Sync via a platform-agnostic marker; on macOS the CoreAudio stream handle is thread-safe.

**File:** `apps/local-bridge/src/main.rs`, `SafeStream` definition (~line 54)

**Acceptance:** Comment present that cites the specific macOS guarantee.

---

### A2 — Remove unused `_threshold` binding

`dsp.rs:296` binds `let _threshold = self.adaptive.record_peak(amplitude)` but never uses the value. Replace with `let _ = self.adaptive.record_peak(amplitude)`.

**File:** `apps/local-bridge/src/dsp.rs`, `detect_peak` method

**Acceptance:** No unused-variable warning for `_threshold`.

---

### A3 — Remove or use `StreamQuery.token`

`main.rs:118-119` defines `StreamQuery { token: Option<String> }` but the handler discards it (`Query(_query)`). Since the WebSocket handler should enforce the token, either:
- Wire the token check into `ws_handler` (compare against `state.pair_token`, reject with 401 on mismatch),
- Or remove the struct and the `Query` extractor if it's truly unused.

Prefer wiring the check (see A6).

**File:** `apps/local-bridge/src/main.rs`, `ws_handler` and `StreamQuery`

**Acceptance:** No unused-field warning; token is checked or struct is removed.

---

### A4 — Cap `interval_history` to prevent unbounded growth

`dsp.rs:219` defines `interval_history: Vec<f64>` that grows unboundedly. Only the first 10 entries are ever consumed (BPH auto-detect at tick 10). Change to `VecDeque<f64>` with a cap of 10.

**File:** `apps/local-bridge/src/dsp.rs`

**Acceptance:** `interval_history` never exceeds 10 entries.

---

### A5 — Fix vacuous test assertion

`dsp.rs:408` asserts `p.ticks.is_empty() || p.ticks.len() > 0` which is always true. Replace with a meaningful assertion.

The test generates 1s of 2 kHz sine audio at 44100 Hz. A reasonable expectation: at least 1 tick detected (the sine envelope will produce periodic peaks). Assert `p.ticks.len() > 0` directly, with a message explaining the expected behaviour.

**File:** `apps/local-bridge/src/dsp.rs`, `test_dsp_pipeline_processes_samples`

**Acceptance:** Test can fail — it must catch a genuinely broken pipeline.

---

### A6 — Enforce WebSocket token auth

`ws_handler` places `/stream` in the public route group. Currently any client can connect. The handler receives `_expected_token: String` but never uses it.

Fix: in `ws_handler`, before calling `on_upgrade`, validate that the query param `token` matches `state.pair_token`. Return 401 if missing or invalid.

**File:** `apps/local-bridge/src/main.rs`, `ws_handler` and `handle_socket`

**Acceptance:** A curl/WebSocket client without a valid token receives 401. A client with `?token=<valid>` connects successfully.

---

### A7 — Add mono downmix to mic capture path

The cpal callback (`start_mic_capture`, `main.rs:273-278`) passes `data: &[f32]` directly to `pipeline.process_samples()` without checking the channel count. If the device delivers stereo interleaved frames, both channels are processed as if mono.

Fix: query the stream config's channel count. If `channels > 1`, mono-mix before passing to the pipeline (same pattern as `play_file` lines 416-419).

**File:** `apps/local-bridge/src/main.rs`, `start_mic_capture` data_fn

**Acceptance:** A stereo mic device produces the same tick count as a mono device on the same source.

---

### A8 — Update spec to match `/status` shape

The original issue 02 spec requested top-level `device_name`, `sample_rate`, `bph`, `lift_angle` in the `/status` response. The implementation nests them under `device: { ... }` via the `DeviceConfig` struct (extracted during a code review refactor of ticket 02).

Update the issue checklist in `02-http-server-token-auth.md` to reflect the actual shape: `GET /status returns { running, device: { device_name, sample_rate, bph, lift_angle }, session_id, total_samples }`.

**File:** `.scratch/local-bridge-mvp/issues/02-http-server-token-auth.md`

**Acceptance:** Spec and implementation agree.

---

### A9 — Extract magic literals to named constants

`dsp.rs` uses `0.25` (refractory fraction, lines 254 and 328) and `28800` (default BPH, lines 11, 246, 252) as bare literals.

- Define `const REFRACTORY_FRACTION: f64 = 0.25` at module level.
- Define `const DEFAULT_BPH: u32 = 28800` at module level.
- Replace all occurrences.

**File:** `apps/local-bridge/src/dsp.rs`

**Acceptance:** No bare `0.25` or `28800` remain in DSP code (except test constants).