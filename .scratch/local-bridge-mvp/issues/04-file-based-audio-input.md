# 04 — File-based audio input with MP3 decoding

**What to build:** The bridge can feed audio from an MP3 or WAV file through the same pipeline as the live mic, enabling offline development without hardware.

**Blocked by:** 03 — cpal audio capture and sample delivery

**Status:** ready-for-agent

- [ ] `--input-file <path>` on the CLI selects file mode instead of cpal
- [ ] File is decoded to raw mono samples matching the bridge's expected format
- [ ] Decoded samples flow through the same interface as cpal (interchangeable source)
- [ ] Playback loops or stops at EOF; configurable via flag
- [ ] Sample rate from the file is used; resampled to 96 kHz if needed
- [ ] `POST /start` and `POST /stop` control file playback the same way they control mic capture
- [ ] Works with at least one provided MP3 test fixture
