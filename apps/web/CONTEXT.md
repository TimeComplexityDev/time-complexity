# Context: apps/web

This directory contains the browser UI for the Time Complexity timegrapher.

## Technology

- React + TypeScript SPA
- Hosted as static site (Vercel / GitHub Pages)
- Connects to local bridge via WebSocket (`ws://127.0.0.1:PORT/stream`)
- Authenticates with a stored pairing token (localStorage, no expiry)

## Domain vocabulary

- **Tick**: A detected escapement beat with timestamp and rate data.
- **Rate (s/day)**: Instantaneous or smoothed rate of the watch movement.
- **Beat error**: Difference between consecutive half-periods, indicating geometry errors.
- **Amplitude**: Proxy for swing amplitude derived from envelope peak or RMS.
- **BPH**: Beats per hour (e.g., 28800, 21600). Auto-detected from observed intervals, with manual override available.
- **Lift angle**: Angle of the escape wheel lift used for amplitude compensation.
- **Session**: A time-bounded recording session with ticks and aggregates.

## Interfaces

- Reads tick events and aggregate updates from the local bridge WebSocket (paired with one-time token).
- Sends control commands (start/stop, BPH, filters) via REST to the local bridge.
- Optionally fetches session history from the FastAPI backend when it exists.

## Current state

Scaffolded; no code yet.
