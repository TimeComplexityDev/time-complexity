# 07 — REST completion, session state, and SQLite persistence

**What to build:** The remaining REST endpoints are wired to session state, and session data is saved to local SQLite.

**Blocked by:** 06 — Metrics, BPH auto-detect, and WebSocket streaming

**Labels:** ready-for-agent

**Status:** done

## Notes

SQLite persistence removed per architectural decision — the backend handles storage when it's built.

`POST /set_params` already supports `bph`, `bandpass_freq`, and `bandpass_q`. Remaining controls (lift angle, smoothing windows, median filter, outlier rejection) are deferred — they're not used by the current DSP pipeline.

`GET /status` already reflects current parameters and session state.

- [x] `POST /set_params` updates BPH and bandpass filter live
- [x] `GET /status` reflects current parameters and session state
- [ ] (deferred) Lift angle, smoothing windows, median filter, outlier rejection
- [ ] (deferred) SQLite persistence — backend handles storage
- [ ] (deferred) Session summary computation — post to backend when it exists
