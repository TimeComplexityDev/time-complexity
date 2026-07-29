# 13 — Refactor: extract SessionState and CaptureConfig from AppState

**What to build:** `AppState` currently bundles ~6 unrelated concerns (auth, device config, session lifecycle, DSP, file mode, BPH override). Extract two sub-structs so each has one reason to change.

**Blocked by:** None (but touches many of the same files as issues 11 and 12)

**Labels:** ready-for-agent

**Status:** pending

## New layout

```rust
struct SessionState {
    session_id: Option<String>,
    audio_stream: SafeStream,
    sample_count: Arc<AtomicU64>,
}

struct CaptureConfig {
    input_file: Option<PathBuf>,
    loop_playback: bool,
}

struct AppState {
    pair_token: String,
    device: DeviceConfig,
    session: SessionState,
    pipeline: Arc<Mutex<DspPipeline>>,
    bph_override: Option<u32>,
    capture_config: CaptureConfig,
}
```

The `running()` method moves to `SessionState`. The file-mode guard in `main()` checks `capture_config.input_file.is_some()`. All handlers access `state.session.audio_stream`, `state.capture_config.input_file`, etc.

## Changes

- Define `SessionState` and `CaptureConfig` in `main.rs`.
- Update `AppState` fields + constructor.
- Update all handlers to use the new field paths.
- No behavioural changes — pure structural refactor.

**File:** `apps/local-bridge/src/main.rs`

## Why not full separate modules

The structs remain in `main.rs` (no new files). They're simple data containers with no methods of their own — the goal is to group fields, not to extract independent subsystems.