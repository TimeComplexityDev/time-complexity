# 08 — Synthetic regression test suite

**What to build:** A deterministic test suite that feeds synthetic audio through the DSP pipeline and asserts on the outputs.

**Blocked by:** 05 — DSP pipeline: bandpass, envelope, peak detection with adaptive threshold

**Status:** ready-for-agent

- [ ] Synthetic click track generator produces a configurable BPH click train with known jitter
- [ ] Click track fed through the full DSP pipeline
- [ ] Assert: tick count within tolerance of expected count for a given duration
- [ ] Assert: mean interval matches nominal interval within tolerance
- [ ] Assert: beat error computed and within range for synthetic asymmetry
- [ ] Assert: amplitude proxy scales with input click amplitude
- [ ] Tests run in CI with no audio hardware required
- [ ] A saved 10-second contact-mic buffer (real watch) used as a regression fixture
