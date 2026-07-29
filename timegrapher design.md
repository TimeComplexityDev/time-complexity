# Time Complexity — Local Timegrapher Design

This document describes the L1 "local bridge" design for the Time Complexity timegrapher application.

Summary
- Single-user, privacy-first architecture: all audio capture and DSP runs locally on the user's Mac.
- Browser UI (static React app) connects to a local bridge binary (localhost) for real-time visualization.
- Derived metrics (ticks, rates, beat error, amplitude, session summaries) can be uploaded to a remote FastAPI backend; raw audio is never uploaded by default.
- Primary hardware target: clip-on contact mic (3.5mm) into Mac.

Goals
- Low-latency, high-resolution tick detection and timestamping.
- Beautiful, video-friendly UI (live graphs, overlays, waveform, spectrogram, gauges).
- Easy local install: single binary on macOS.
- Extensible: configurable filters, averaging windows, export, and optional cloud sync for metrics.

High-level architecture
- Local bridge (binary running on the user's Mac): audio capture (CoreAudio/PortAudio), DSP, REST control endpoints, WebSocket stream for live events.
- Frontend: React + TypeScript single-page app (hosted as static site on Vercel/GitHub Pages) that connects to ws://127.0.0.1:PORT.
- Backend (optional): FastAPI server for inventory, scraping notifications, and session summary ingestion. Single-user auth (API token).
- Local persistence: SQLite for session logs; session summaries POST to FastAPI if configured.

Why local DSP
- Microphone access and privacy: audio never leaves the machine unless explicitly requested.
- Low latency required for a responsive UI and accurate timing.
- Contact mic + local DSP increases SNR and accuracy compared to browser-only mic capture in many cases.

Language & packaging
- Recommended: Rust for safety and portability, or C++ for faster reuse of existing tg code.
- Deliverable: a macOS binary that binds to 127.0.0.1 and exposes REST + WebSocket.

Audio capture & sample-rate
- Prefer highest supported sample rate; target default: 96 kHz (fallback 48 kHz).
- Bit-depth: 16-bit default; 24-bit if the device supports it.
- Use absolute monotonic clock + sample index to compute precise timestamps.

DSP pipeline (concrete)
1. Preprocessing
   - Mono mix, biquad band-pass (default band 800 Hz – 6000 Hz), configurable.
2. (Optional) Template calibration
   - Capture a short tick snippet to build a matched-filter template.
3. Matched filtering / envelope
   - Matched filter (time-reversed template) or envelope detection (Hilbert + lowpass).
4. Peak detection
   - Threshold + refractory period (~0.25 × nominal_half_period).
5. Sub-sample timing
   - Parabolic interpolation on peak and neighbors to estimate fractional sample offset.
   - Peak time = (sample_index + δ) / sample_rate
6. Beat pairing & BPH
   - Use BPH to compute nominal interval: nominal_interval = 3600 / BPH
   - Pair ticks for half-periods, compute beat error = half1 - half2
7. Metrics
   - instantaneous_interval, seconds_per_day (s/day), beat_error, amplitude proxy (matched-filter peak or RMS)
8. Averaging / smoothing
   - Short moving average (window 1–30s), long EWMA (tau configurable, default 600s), median filter and outlier rejection option

Sub-sample interpolation (parabolic)
- Use three samples around the peak: y[-1], y0, y[+1]
- δ = (y[-1] - y[+1]) / (2*(y[-1] - 2*y0 + y[+1]))
- fractional_time = δ / sample_rate

Filters and UI controls
- Bandpass (low, high)
- Short window moving average (slider)
- Long-term EWMA tau (slider)
- Median filter toggle
- Outlier rejection threshold
- BPH and lift angle controls (auto-detect BPH option)

Local bridge API (127.0.0.1 only)
REST endpoints (JSON)
- GET /status
  - Returns: { running, device_name, sample_rate, bph, lift_angle, session_id }
- GET /devices
  - Returns list of available audio inputs
- POST /start { bph?, lift_angle?, sample_rate? }
- POST /stop
- POST /set_params { bph, lift_angle, filters... }
- POST /pair { token }

WebSocket stream
- ws://127.0.0.1:PORT/stream
- Message types (JSON):
  - Tick event:
    {
      "type": "tick",
      "session_id": "uuid",
      "tick_index": 1234,
      "timestamp_iso": "2026-07-29T22:07:30.123456Z",
      "timestamp_monotonic": 98765.4321,
      "sample_index": 112345678,
      "fractional_offset": -0.24,
      "interval_s": 0.5,
      "rate_spd": -2.3,
      "amplitude": 0.876,
      "peak_window": [ ... ]
    }
  - Aggregate update (every second):
    {
      "type": "aggregate",
      "session_id":"uuid",
      "time": "2026-07-29T22:07:31Z",
      "instant_rate_spd": -2.3,
      "short_avg_spd": -1.8,
      "long_ewma_spd": -1.6,
      "beat_error_s": 0.00012,
      "amplitude": 2300
    }

Security
- Bind only to 127.0.0.1.
- Require a one-time pairing token (CLI prints token or you can set via config) to authorize browser connections.
- Do not expose service on LAN unless explicitly configured.

Local persistence & cloud sync
- Local SQLite: table for ticks (tick_index, timestamp_iso, interval_s, rate_spd, beat_error_s, amplitude).
- On session end create a session summary JSON with mean, stdev, median, duration.
- If FastAPI URL + token configured, POST session summary to remote server.
- Never upload raw audio by default.

UI design notes (video-friendly)
- Live rate graph: raw instantaneous points + smoothed curve; adjustable time scale
- Large numeric HUD: rate (s/day), beat error (ms), amplitude
- Waveform pane with detected peaks and matched-filter output
- Spectrogram view (optional)
- Controls: BPH auto-detect/manual, lift angle, filters, smoothing windows, start/stop
- Session recording/export (CSV of per-tick data)
- Overlay mode: simplified HUD for screen recording/videos

Default runtime parameters
- sample_rate: try 96000, fallback 48000
- bandpass: 800–6000 Hz
- matched filter length: 2–6 ms
- refractory period: 0.25 × nominal_half_period
- short-term window: 10 s
- long-term EWMA tau: 600 s

Testing & validation
- Unit tests: parabolic interpolation, timestamp arithmetic, rate conversion
- Synthetic test input: generated click track with known jitter to validate interval detection and sub-sample interpolation accuracy
- Compare against commercial timegrapher on same watch to validate offsets and variance
- Compute Allan deviation to characterize stability/noise floor

Milestones (MVP roadmap)
- Week 0: finalize language choice and repo layout
- Week 1: MVP local bridge: audio capture + matched-filter detection + console tick logging
- Week 2: Add REST + WebSocket stream; simple React demo consumer for live plot
- Week 3: UI features: waveform pane, smoothing windows, session logging and CSV export
- Week 4: Packaging (macOS binary), pairing token, optional FastAPI session uploads
- Week 5: Tuning, docs, calibration UI, video overlay templates

Integration with existing open-source timegrapher (vacaboja/tg)
- vacaboja/tg is a mature C/C++ tool that implements many of the features described here.
- Options:
  - Reuse DSP code from tg (C++) and wrap in a local bridge with REST/WS endpoints.
  - Reimplement algorithms (matched filtering, interpolation, beat pairing) in Rust for easier future WASM/portability.
  - Hybrid: port key algorithms and validate results against tg.

CLI examples
- Start with device and default params:
  timebridge --port 9001 --device "Built-in Input" --sample-rate 96000 --bph 28800 --lift-angle 52 --pair-token ABC123

Data export CSV columns
- tick_index, timestamp_iso, interval_s, rate_spd, beat_error_s, amplitude

Next actions (pick one)
- A) Scaffold the local bridge project skeleton (Rust or C++) with audio capture + tick console logging.
- B) Produce the full Wireframe + REST + WS API spec and a small React demo consumer.
- C) Fetch and analyze the vacaboja/tg repository to extract DSP algorithms for reuse or porting.
- D) Implement and unit-test the parabolic interpolation micro-module.
- E) Generate macOS packaging instructions and a draft installer pipeline.

Decide which of the next actions (A/B/C/D/E) you want me to start and confirm language for the binary (Rust or C++).