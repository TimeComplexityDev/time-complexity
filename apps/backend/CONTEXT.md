# Context: apps/backend

This directory contains the optional FastAPI backend for session storage and metrics ingestion.

## Technology

- FastAPI (Python)
- SQLite for local persistence (mirrored in the local bridge)
- Single-user auth via API token

## Domain vocabulary

- **Session summary**: Aggregated metrics (mean, stdev, median, duration) uploaded at session end.
- **Inventory**: Watch configuration and calibration data.
- **Scraping notifications**: Alerts for watch service releases or market data.

## Interfaces

- Accepts POST of session summaries from the local bridge.
- Optionally serves the React SPA or an API proxy.

## Current state

Scaffolded; deferred. The backend will be built later to support watch inventory (acquisition, repairs, parts, timegrapher data). For now, focus is on the local bridge and web UI. Session summaries are stored locally by the local bridge; backend upload is a future integration point.
