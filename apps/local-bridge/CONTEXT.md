# Context: apps/local-bridge

This directory contains the local macOS bridge binary for audio capture and DSP.

## Technology

- Rust
- Binds to `127.0.0.1` only
- Exposes REST + WebSocket endpoints
- Audio capture via `cpal` (CoreAudio on macOS, mono, up to 96 kHz)
- DSP algorithms reimplemented from scratch in Rust (no C/C++ FFI)

## Domain vocabulary

- **Matched filter**: Time-reversed template used for tick detection (deferred — MVP uses envelope detection).
- **Parabolic interpolation**: Sub-sample peak timing estimation.
- **Half-period**: `1800 / BPH`, the time between consecutive ticks (tic→tok or tok→tic). Each beat produces two ticks.
- **BPH**: Beats per hour (e.g., 28800, 21600). Auto-detected from observed half-period intervals, with manual override available.
- **Refractory period**: Minimum gap between detected ticks (`0.25 × half-period`).
- **Adaptive threshold**: Two thresholds derived from observed tic/tok amplitudes — one for each half-period direction.
- **Average window**: 30 s rolling average of tick rates.
- **Beat error**: `|tic_interval - tok_interval|`, averaged over the last 10 consecutive pairs. Reported in seconds.

## Interfaces

- REST: `/status`, `/devices`, `/start`, `/stop`, `/set_params`, `/pair`
- WebSocket: `/stream` for tick events and aggregate updates

## Security model

- Binds to `127.0.0.1` only. Use `--allow-remote` for LAN access (future).
- Requires a one-time pairing token for WebSocket and REST authorization.
- Token is generated on first run and printed to stdout / stored in config. Reset via `--reset-pairing`.
- Backend is not in the critical path — local bridge runs independently.
