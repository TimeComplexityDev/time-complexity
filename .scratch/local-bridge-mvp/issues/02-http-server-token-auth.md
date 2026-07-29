# 02 — HTTP server, token auth, device enumeration, /start/ /stop

**What to build:** The bridge serves a REST API on `127.0.0.1:PORT` with token authentication and device control. No audio capture yet.

**Blocked by:** 01 — Rust project skeleton and CLI

**Labels:** ready-for-agent

**Status:** done

- [x] HTTP server listens on `127.0.0.1:PORT`
- [x] Every request requires `Authorization: Bearer <pair-token>` header; missing/invalid returns 401
- [x] `POST /pair` accepts a token and stores it as the active pairing token
- [x] `GET /devices` returns the list of available audio input device names
- [x] `GET /status` returns `{ running, device_name, sample_rate, bph, lift_angle, session_id }`
- [x] `POST /start` begins a session (no audio yet, just transitions state)
- [x] `POST /stop` ends the session and returns a confirmation
