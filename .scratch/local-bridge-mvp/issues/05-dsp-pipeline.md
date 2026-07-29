# 05 — DSP pipeline: bandpass, envelope, peak detection with adaptive threshold

**What to build:** Raw audio samples from cpal or file are transformed into timestamped tick detections.

**Blocked by:** 03 — cpal audio capture and sample delivery

**Status:** ready-for-agent

- [ ] Samples pass through a configurable biquad bandpass (default 800–6000 Hz)
- [ ] Hilbert envelope computed on the filtered signal
- [ ] Envelope lowpassed to a smooth amplitude curve
- [ ] Peaks detected above a dynamic threshold
- [ ] Adaptive per-direction threshold derived from observed tic/tok amplitudes (warm-up window)
- [ ] Refractory period enforced at `0.25 × nominal_half_period` once BPH is known
- [ ] Each detection carries a timestamp, sample index, and sub-sample fractional offset
- [ ] Tick detections logged to console with timestamps and intervals
- [ ] DSP module is pure Rust with no I/O, so it can be unit-tested independently
