# ADR 0002: Local bridge implemented in Rust, DSP reimplemented from scratch

## Status

Accepted

## Context

The Time Complexity repo contains a local bridge binary (`apps/local-bridge`) responsible for audio capture, DSP, and live event streaming. The original design proposed Rust or C++ and considered reusing `vacaboja/tg` C/C++ DSP code, using a hybrid port, or reimplementing from scratch.

## Decision

- Implement `apps/local-bridge` in Rust.
- Reimplement all DSP algorithms from scratch in Rust rather than integrating C/C++ via FFI or wrapping `vacaboja/tg`.
- Use Rust-native DSP libraries where appropriate.

## Consequences

- Avoids C++ FFI complexity and packaging overhead.
- Keeps the macOS binary dependency-light and portable.
- Gives direct ownership of the DSP pipeline, making tuning and correctness verification straightforward.
- Adds implementation effort, but the algorithms involved (matched filtering, parabolic interpolation, beat pairing) are compact and suitable for a solo Rust learner.
