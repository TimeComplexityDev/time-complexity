# 03 — cpal audio capture and sample delivery

**What to build:** The bridge captures real microphone audio via cpal and delivers raw mono samples to a test sink.

**Blocked by:** 02 — HTTP server, token auth, device enumeration, /start/ /stop

**Status:** ready-for-agent

- [ ] `cpal` integrated and builds on macOS
- [ ] `POST /start` triggers a cpal stream from the selected device
- [ ] Target sample rate: 96 kHz; fallback to 48 kHz if the device doesn't support 96 kHz
- [ ] 16-bit default; logs actual format negotiated with CoreAudio
- [ ] Mono mix delivered as a contiguous sample stream
- [ ] Sample count and buffer health logged to console while running
- [ ] `POST /stop` halts the cpal stream cleanly
- [ ] Running state reflected in `GET /status`
