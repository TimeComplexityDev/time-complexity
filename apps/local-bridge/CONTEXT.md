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
- **Nominal interval**: `3600 / BPH`, the expected half-period duration.
- **BPH**: Beats per hour (e.g., 28800, 21600). Auto-detected from observed intervals, with manual override available.
- **Refractory period**: Minimum gap between detected ticks (`0.25 × nominal_half_period`).
- **Adaptive threshold**: Two thresholds derived from observed tic/tok amplitudes — one for each half-period direction.
- **Short window**: 1–30 s moving average.
- **Long EWMA**: Exponentially weighted moving average (default tau 600 s).

## Interfaces

- REST: `/status`, `/devices`, `/start`, `/stop`, `/set_params`, `/pair`
- WebSocket: `/stream` for tick events and aggregate updates

## Security model

- Binds to `127.0.0.1` only. Use `--allow-remote` for LAN access (future).
- Requires a one-time pairing token for WebSocket and REST authorization.
- Token is generated on first run and printed to stdout / stored in config. Reset via `--reset-pairing`.
- Backend is not in the critical path — local bridge runs independently.
