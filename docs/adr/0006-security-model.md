# ADR 0006: Security model — pairing token, 127.0.0.1, no backend dependency for bridge

## Status

Accepted

## Context

The local bridge exposes REST and WebSocket on localhost. The web UI is served from a remote origin (Vercel/GitHub Pages). Browsers allow cross-origin connections to `ws://127.0.0.1`, so any page the user visits could reach the bridge if CORS were permissive.

Alternative designs considered:
- Backend-brokered auth (Google OAuth + static bridge secret): rejected because it makes the bridge depend on the backend being online, adding latency and a single point of failure for local use.
- No auth at all: rejected because a malicious website could control the bridge.

## Decision

- Local bridge binds to `127.0.0.1` only. `--allow-remote` is a future opt-in for LAN/tablet use.
- WebSocket and REST connections require a one-time pairing token.
- Bridge generates a token on first run (or reads from config). The user enters it once in the web UI.
- UI stores the token in `localStorage` with no expiry. Reconnects are automatic.
- Token is only reset via `--reset-pairing` or config deletion.
- Backend is not required for local bridge operation. It remains a separate, independently deployable service.

## Consequences

- Single-user, low-friction: one manual entry, then auto-reconnect.
- Protects against drive-by localhost attacks from random websites.
- No CORS complexity — the bridge rejects unauthorized WebSocket upgrades at the token check, not via CORS headers.
- Backend remains optional and decoupled. When it exists, it can ingest exported session summaries; it is not in the critical path for timegrapher usage.
