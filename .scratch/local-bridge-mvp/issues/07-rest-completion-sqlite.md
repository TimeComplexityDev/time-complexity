# 07 — REST completion, session state, and SQLite persistence

**What to build:** The remaining REST endpoints are wired to session state, and session data is saved to local SQLite.

**Blocked by:** 06 — Metrics, BPH auto-detect, and WebSocket streaming

**Status:** ready-for-agent

- [ ] `POST /set_params` updates BPH, lift angle, bandpass bounds, and smoothing windows live
- [ ] `POST /set_params` applies median filter toggle and outlier rejection threshold
- [ ] `GET /status` reflects current parameters and session state
- [ ] On session end, a session summary JSON is written to local SQLite
- [ ] SQLite schema includes ticks table: `tick_index, timestamp_iso, interval_s, rate_spd, beat_error_s, amplitude`
- [ ] Session summary includes mean, stdev, median, duration
- [ ] Session history is queryable for future backend ingestion
- [ ] No raw audio is written; only derived metrics are persisted
