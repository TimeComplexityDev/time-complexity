# ADR 0009: Evaluation aggregates — computed once at completion

## Status

Accepted

## Context

An Evaluation (Watch → Evaluation → PositionReading) is a session of measuring a watch in one or more positions. At completion, we need summary statistics: average rate, maximum positional error, average beat error, and average amplitude.

Two approaches: (A) compute these on-the-fly from PositionReadings every time the Evaluation is read, or (B) compute them once at completion time and store them on the Evaluation row.

## Decision

Option B: compute and store at completion. The Evaluation row gains denormalised aggregate fields that are set exactly once when `status` transitions to `complete` and never updated again.

Stored fields:

| Field | Source |
|---|---|
| `avg_rate_spd` | mean of PositionReading rates |
| `max_delta_rate_spd` | max rate − min rate (positional error) |
| `avg_beat_error_ms` | mean of PositionReading beat errors |
| `avg_amplitude` | mean of PositionReading amplitudes |

## Rationale

- **Snapshot immutability.** An Evaluation is a historical record. If a PositionReading is later corrected (e.g., a typo), the Evaluation summary should not drift — it should reflect what was observed at the time of measurement.
- **Read performance.** "Show me the latest evaluation for this watch" is a frequent query. Recomputing aggregates on every read, even over 1–6 rows, is unnecessary.

## Consequences

- Evaluations are write-once after completion; aggregates never go stale.
- If we later add new aggregate metrics (e.g., weighted rates), old evaluations won't have them — but that's acceptable because re-computing would change history.