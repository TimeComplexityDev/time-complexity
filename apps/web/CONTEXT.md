# Context: apps/web

Browser UI for the Time Complexity timegrapher — a React SPA that connects to the local bridge for live capture and to the backend for persisted results.

## Technology

- React + TypeScript SPA
- Hosted as static site (Vercel / GitHub Pages)
- Connects to local bridge via WebSocket (`ws://127.0.0.1:PORT/stream`)
- Authenticates with a stored pairing token (localStorage, no expiry)

## Domain vocabulary

**Watch**:
A physical watch movement being evaluated. Lightweight entity with a name and optional notes. One Watch can have multiple Evaluations over time. Top-level navigation entity: home screen shows watch list, each watch has its evaluation history and a "New Evaluation" button.
_Avoid_: Movement, caliber (unless the user explicitly distinguishes them)

**Evaluation**:
An assessment of a watch's timing performance, consisting of one or more Position Readings. The top-level unit a watchmaker creates and names. Default name is `"{Watch name} — {date}"`, editable inline. Lives in states: `draft` → `in_progress` → `complete`. Auto-completes when all 5 positions have a successful Position Reading; also has a manual "Finish" button that works from 1 position onward. BPH is auto-detected during the first Position Reading and locked for the entire Evaluation (override available). Computes multi-position aggregates at completion: average rate across all positions, maximum positional error (delta between best and worst position).

**Position Reading**:
A single data collection at one watch orientation, backed by one bridge session. Tagged with a position name (dial up, crown down, etc.). Lives in states: `recording` → `complete`, or `recording` → `failed`. Created when the user selects a position and clicks "Start" — enters `recording`. On bridge disconnect mid-reading, transitions to `failed` (shows "Retry" button — retrying replaces the failed reading for that position). On successful stop, transitions to `complete` with per-position statistics (rate, beat error, amplitude). During capture the UI shows both a live tick stream (verifying contact mic placement) and aggregate gauges (instant rate, short/long averages, beat error).
_Avoid_: Position capture, position sample, reading

**Positions**:
Hardcoded set of standard orientations: dial up, dial down, crown down, crown left, crown right. Selected from a dropdown when creating a Position Reading. No free-text positions — ensures reliable cross-reading comparison.

## Implementation

- **Persistence (MVP)**: localStorage with flat keys (`watches`, `evaluations`, `position_readings`). Each key stores a serialized array or map. Full dataset is replaced on any write. Planned migration to backend Postgres (see ADR 0008).

## Interfaces

- Reads tick events and aggregate updates from the local bridge WebSocket (paired with one-time token).
- Sends control commands (start/stop, BPH, filters) via REST to the local bridge.
- Optionally fetches session history from the FastAPI backend when it exists.