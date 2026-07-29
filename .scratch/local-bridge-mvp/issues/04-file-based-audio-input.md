# 04 — File-based audio input with MP3 decoding

**What to build:** The bridge can feed audio from an MP3 or WAV file through the same pipeline as the live mic, enabling offline development without hardware.

**Blocked by:** 03 — cpal audio capture and sample delivery

**Labels:** ready-for-agent

**Status:** done

- [x] `--input-file <path>` on the CLI selects file mode instead of cpal
- [x] File decoded to raw mono samples; sample rate and channel count reported in status
- [x] Decoded samples counted through the same `sample_count` mechanism as cpal (interchangeable at the `StreamBackend` enum level)
- [x] Playback stops at EOF; `--loop-playback` flag enables looping
- [x] File's native sample rate is preserved and reported in `/status`
- [x] `POST /start` and `POST /stop` control file playback identically to mic capture
- [x] Test fixture committed: `test-fixtures/mechanical_watch_1.mp3` (44100 Hz, 1 ch, ~61s of watch ticking)
