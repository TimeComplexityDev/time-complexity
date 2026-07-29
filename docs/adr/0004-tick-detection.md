# ADR 0004: Tick detection — envelope detection with adaptive per-direction threshold

## Status

Accepted

## Context

A mechanical watch produces two impulses per beat (tic and tok). In practice one is often louder — due to the watch not being in beat, lift angle asymmetry, or mic geometry — so a single global threshold risks missing the quieter side. Missed ticks break half-period pairing and make beat error meaningless.

Matched filter was considered but deferred; it requires calibration and template management.

## Decision

- Use envelope detection (Hilbert + lowpass) as the primary tick detector for MVP.
- After an initial warm-up, compute two adaptive thresholds: one for the louder half-period direction, one for the quieter.
- No user-facing calibration step. Thresholds are derived automatically from observed tick amplitudes.
- Fall back to a single global threshold during the warm-up period until enough samples are collected.

## Consequences

- Works out of the box with a contact mic, no setup required.
- Handles asymmetric tic/tok amplitude as long as both sides exceed their respective thresholds.
- Requires a warm-up window (a few seconds of beats) before full accuracy. During warm-up, quieter ticks may be missed.
- If both sides drop too low (very worn movement, poor mic contact), the system should surface a degraded-signal indicator in the UI rather than silently produce garbage metrics.
