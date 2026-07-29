# 08 — Synthetic regression and accuracy test suite

**What to build:** Extend the existing test suite with beat error assertions, jitter accuracy validation, amplitude scaling checks, and a `mechanical_watch_1.mp3` regression fixture.

**Blocked by:** 05 — DSP pipeline

**Labels:** ready-for-agent

**Status:** pending

## Merged from tickets 08 and 09

- [ ] Assert beat error: feed a click train with known beat error (e.g. 1.2 ms) through the DSP, verify the measured beat error matches within tolerance
- [ ] Assert amplitude scaling: verify that changing the click amplitude scales the detected amplitude proportionally
- [ ] Add jitter parameter to `generate_test_watch.py`: each tick's interval deviates from the nominal by a Gaussian random amount (e.g. ±50 µs)
- [ ] Assert parabolic interpolation accuracy: process a jittered click train, verify the tick-to-tick residual error is significantly below the raw integer-sample error
- [ ] Add `mechanical_watch_1.mp3` regression test — feed the real recording through the DSP, assert tick count and mean interval match known-good values
- [ ] Tests run in CI with no audio hardware required