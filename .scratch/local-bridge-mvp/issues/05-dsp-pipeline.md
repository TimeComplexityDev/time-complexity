# 05 — DSP pipeline: bandpass, envelope, peak detection with adaptive threshold

**What to build:** Raw audio samples from cpal or file are transformed into timestamped tick detections.

**Blocked by:** 03 — cpal audio capture and sample delivery

**Labels:** ready-for-agent

**Status:** done

- [x] Samples pass through a configurable biquad bandpass (default center=2000 Hz, Q=0.4; configurable via `set_bandpass()`)
- [x] Envelope computed via abs() + lowpass at 200 Hz (non-Hilbert; sufficient for impulse detection)
- [x] Envelope lowpassed to a smooth amplitude curve
- [x] Peaks detected above dynamic adaptive thresholds (per-direction alternating bins)
- [x] Adaptive per-direction threshold derived from observed tic/tok amplitudes (0.5s warm-up window, 8-sample window)
- [x] Refractory period enforced at `0.25 × nominal_half_period`; BPH auto-detected from first 10 intervals using nearest common BPH (18000/19800/21600/25200/28800/36000)
- [x] Each detection carries sample index, sub-sample fractional offset (parabolic interpolation, clamped to [-0.5, 0.5]), amplitude, interval, and absolute timestamp
- [x] DSP module is pure Rust with no I/O (no println! in hot path; logging delegated to tests/consumers)
- [x] 7 unit tests covering biquad stability, parabolic peak, BPH detection, and pipeline sanity
- [x] Synthetic test generator: `tools/generate_test_watch.py` (Requirement B from architecture plan)
- [x] Test fixture: `test-fixtures/test_21600_drift+12s_be1.2ms.wav`
