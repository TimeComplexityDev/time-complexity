# 06 — Metrics, BPH auto-detect, and WebSocket streaming

**What to build:** Tick detections are paired into half-periods, metrics are computed, and live data is streamed to the browser with pairing token auth.

**Blocked by:** 05 — DSP pipeline: bandpass, envelope, peak detection with adaptive threshold

**Labels:** ready-for-agent

**Status:** done

- [x] First N half-periods auto-detect BPH by dominant period estimation (in DSP pipeline)
- [x] BPH locks after convergence and becomes the nominal interval for pairing
- [x] Manual override available via `POST /set_params` REST endpoint and WebSocket `bph` command
- [x] Ticks paired for half-periods; beat error computed as `|half1 - half2|` (in MetricsEngine)
- [x] Rate (s/day), amplitude proxy, and EWMA smoothing computed per tick
- [x] Short window moving average (default ~60 samples / ~10s) and long EWMA (default tau 600 s)
- [x] Aggregate update emitted every second: `instant_rate_spd`, `short_avg_spd`, `long_ewma_spd`, `beat_error_s`, `amplitude`
- [x] Tick event and aggregate JSON formats match the design doc schema
- [x] WebSocket at `/stream` with token auth (checked via handler); `POST /set_params` for BPH/bandpass override
- [x] Reconnects succeed without re-pairing (token stored in localStorage)
