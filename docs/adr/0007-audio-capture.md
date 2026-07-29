# ADR 0007: Rust audio capture via cpal

## Status

Accepted

## Context

The local bridge needs to capture audio from the contact mic at up to 96 kHz on macOS. We need a Rust library that wraps CoreAudio and provides a raw sample stream to the DSP pipeline.

## Decision

- Use `cpal` for audio capture.
- Target highest supported sample rate (default 96 kHz, fallback 48 kHz), 16-bit default.
- Capture mono mix; bandpass filtering happens downstream in the DSP pipeline.

## Consequences

- Cross-platform by default, but macOS is the only target for now.
- `cpal` handles CoreAudio device enumeration, format negotiation, and stream management.
- Adds a well-maintained dependency with a stable API.
