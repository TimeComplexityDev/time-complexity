# Local Bridge MVP — Spec

## Problem Statement

A mechanical watch produces visible rate changes, beat error, and amplitude shifts that can only be measured with precision audio timing. The user needs a local binary that captures audio from a contact mic, detects ticks with sub-sample precision, computes movement metrics in real time, and streams them to a web UI for visualization and control.

## Solution

Build `apps/local-bridge` as a Rust binary that captures audio via the contact mic at up to 96 kHz, runs an envelope-detection DSP pipeline with adaptive per-direction thresholds, and exposes a REST + WebSocket API on `127.0.0.1`. The user enters a one-time pairing token in the web UI, which then streams live tick events and aggregate updates and sends control commands back. Session summaries are persisted to local SQLite for future backend ingestion.

## User Stories

1. As a watchmaker, I want to start a recording session from my browser, so that I can measure a watch's current rate while I work.
2. As a watchmaker, I want to see live instantaneous rate (s/day) update every tick, so that I can judge whether the movement is running fast or slow.
3. As a watchmaker, I want to see a smoothed short-term rate alongside the raw rate, so that I can compare instant behavior with short-term trend.
4. As a watchmaker, I want to see a long-term EWMA rate, so that I can judge stability over minutes.
5. As a watchmaker, I want to see beat error (s) update in real time, so that I can assess escapement geometry.
6. As a watchmaker, I want to see an amplitude proxy update in real time, so that I can judge swing amplitude without a separate measuring tool.
7. As a watchmaker, I want the BPH to auto-detect from the first few beats, so that I don't have to look it up and enter it manually.
8. As a watchmaker, I want to override the BPH manually at any time, so that I can correct auto-detect errors or lock to a known movement value.
9. As a watchmaker, I want to enter a pairing token once in the browser, so that I can connect to the bridge without repeated setup.
10. As a watchmaker, I want the bridge to bind to localhost only by default, so that no remote device can reach it.
11. As a watchmaker, I want a one-time pairing token check on every WebSocket/REST request, so that a random website I visit cannot control my bridge.
12. As a watchmaker, I want to see a list of available audio input devices, so that I can select the correct contact mic.
13. As a watchmaker, I want to start/stop sessions from the browser, so that I don't need to touch the terminal.
14. As a watchmaker, I want to adjust bandpass filter bounds from the UI, so that I can tune for different watch tick pitches.
15. As a watchmaker, I want to adjust the short-term moving average window, so that I can trade responsiveness for smoothness.
16. As a watchmaker, I want to adjust the long-term EWMA tau, so that I can change how quickly old data fades.
17. As a watchmaker, I want to toggle median filter and outlier rejection, so that I can handle noisy environments.
18. As a watchmaker, I want to adjust lift angle from the UI, so that amplitude compensation stays correct.
19. As a watchmaker, I want session summaries persisted to local SQLite, so that my history is preserved across bridge restarts.
20. As a watchmaker, I want the bridge to print its pairing token and status on startup, so that I can set it up without opening the web UI first.
21. As a watchmaker, I want to reset the pairing token from the CLI, so that I can revoke access if needed.
22. As a watchmaker, I want tick events to carry all data needed to reconstruct the session offline, so that I can analyze them later.
23. As a watchmaker, I want aggregate updates every second, so that I can see a live rolling rate graph without processing raw ticks in the UI.
24. As a developer, I want the DSP pipeline to expose a single tested seam, so that I can swap in a matched filter later without changing the rest of the bridge.
25. As a developer, I want the bridge binary to be a single macOS deliverable with no dynamic runtime dependencies, so that distribution is straightforward.

## Implementation Decisions

- **Component:** `apps/local-bridge` shipped as a single Rust binary.
- **Audio capture:** `cpal` for CoreAudio access on macOS. Target 96 kHz mono, 16-bit; fallback to 48 kHz.
- **Tick detection:** Envelope detection (Hilbert transform + lowpass) with adaptive per-direction threshold. No user calibration required.
- **Sub-sample timing:** Parabolic interpolation on the envelope peak.
- **BPH handling:** Auto-detect from first N half-periods, then lock. Manual override via REST and WebSocket. No backend dependency.
- **Metrics:** instantaneous interval, rate (s/day), beat error, amplitude proxy. Short window moving average (default 10 s), long EWMA (default tau 600 s), optional median/outlier rejection.
- **API:** REST (`/status`, `/devices`, `/start`, `/stop`, `/set_params`, `/pair`) and WebSocket (`/stream` for tick events and aggregate updates).
- **Security:** Bind `127.0.0.1` only. Require one-time pairing token. Browser stores token in `localStorage` indefinitely.
- **Persistence:** Local SQLite for tick logs and session summaries. Backend upload is explicitly deferred.
- **Packaging:** Single static macOS binary. Future: notarization and installer.

## Testing Decisions

- Test the DSP pipeline as a single pure-Rust module. Inputs are sample buffers; outputs are tick events. No I/O in the module itself.
- Unit-test: parabolic interpolation with known peak positions, bandpass filter frequency response, timestamp arithmetic, rate conversion.
- Integration-test: synthetic click track with known jitter fed through the full pipeline; assert tick count, interval accuracy, and beat error within tolerance.
- Regression-test: a saved 10-second contact-mic buffer captured from a real watch, used to lock in behavior across DSP changes.

## Out of Scope

- Matched filter calibration mode (deferred post-MVP)
- Spectrogram view and waveform pane (post-MVP)
- Session CSV export (post-MVP)
- Video overlay mode (post-MVP)
- FastAPI backend for session upload or watch inventory (post-MVP)
- LAN exposure (`--allow-remote`) (post-MVP)
- Automated macOS notarization and installer pipeline (post-MVP)
- Android / iOS / Windows / Linux (macOS only for MVP)

## Further Notes

- This spec intentionally omits UI decisions; those live under `apps/web/CONTEXT.md` and will be specified separately.
- All ADRs referenced above are under `docs/adr/` and should be preserved alongside this spec.
