# 09 — Complex synthetic test files with jitter and varying drift

**What to build:** Extend the synthetic test generator to produce audio where the tick-to-tick interval varies slightly (simulating real-world measurement noise), so the parabolic interpolation's sub-sample accuracy is actually exercised and can be validated against ground truth.

**Blocked by:** 05 — DSP pipeline

**Labels:** ready-for-agent

**Status:** pending

- [ ] Add jitter parameter to `generate_test_watch.py`: each tick's interval deviates from the nominal by a random amount (e.g. ±X µs Gaussian)
- [ ] Generate a test file with 21600 BPH, +12 s/day drift, 1.2 ms beat error, and ±50 µs per-tick jitter
- [ ] Process through DSP pipeline
- [ ] Assert that the mean drift and beat error are recovered within tolerance despite jitter
- [ ] Assert that the per-tick residual error after parabolic interpolation is significantly lower than the raw integer-sample error would be