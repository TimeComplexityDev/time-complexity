# 03 — cpal audio capture and sample delivery

**What to build:** The bridge captures real microphone audio via cpal and delivers raw mono samples to a test sink.

**Blocked by:** 02 — HTTP server, token auth, device enumeration, /start/ /stop

**Labels:** ready-for-agent

**Status:** done

- [x] `cpal` integrated and builds on macOS
- [x] `POST /start` triggers a cpal stream from the default input device
- [x] Attempts 96 kHz with F32 format; falls back to default device config
- [x] Logs the actual format negotiated with CoreAudio (sample rate, channels, format)
- [x] Mono stream delivered as a contiguous `&[f32]` sample buffer
- [x] Sample count accumulated and reflected in `GET /status`
- [x] `POST /stop` drops the stream handle, halting capture cleanly
- [x] Running state reflected in `GET /status`
