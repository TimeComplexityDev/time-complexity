# 06 — Metrics, BPH auto-detect, and WebSocket streaming

**What to build:** Tick detections are paired into half-periods, metrics are computed, and live data is streamed to the browser with pairing token auth.

**Blocked by:** 05 — DSP pipeline: bandpass, envelope, peak detection with adaptive threshold

**Status:** ready-for-agent

- [ ] First N half-periods auto-detect BPH by dominant period estimation
- [ ] BPH locks after convergence and becomes the nominal interval for pairing
- [ ] Manual override available via REST and WebSocket `set_params`
- [ ] Ticks paired for half-periods; beat error computed as `half1 - half2`
- [ ] Rate (s/day), amplitude proxy, and EWMA smoothing computed per tick
- [ ] Short window moving average (default 10 s) and long EWMA (default tau 600 s) emitted
- [ ] Aggregate update emitted every second: `instant_rate_spd`, `short_avg_spd`, `long_ewma_spd`, `beat_error_s`, `amplitude`
- [ ] Tick event and aggregate JSON formats match the design doc schema
- [ ] WebSocket upgrade requires valid pairing token
- [ ] Reconnects succeed without re-pairing
