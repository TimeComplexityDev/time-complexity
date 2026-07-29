# ADR 0005: BPH auto-detection with manual override

## Status

Accepted

## Context

The local bridge needs a BPH value to compute nominal half-period intervals and pair ticks for beat error. BPH can be inferred from observed tick intervals, but the user may also want to set it manually. In the future, a watch inventory database could supply the movement's nominal BPH as a pre-filled default.

## Decision

- On session start, auto-detect BPH from the first N observed half-period intervals.
- Once converged, lock the auto-detected BPH and use it for all subsequent pairing.
- Expose BPH as a settable parameter via REST and WebSocket so the user can override at any time.
- When the backend/watch database exists, it can pre-populate the BPH field, but the local bridge will treat it as a manual override (still adjustable).

## Consequences

- Zero-config start: the watch doesn't need to be pre-registered.
- Manual override recovers from auto-detect errors (e.g., if early ticks are noisy).
- Future backend integration is straightforward — just pre-fill a field that's already writable.
