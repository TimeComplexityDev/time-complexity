# ADR 0008: Web UI data model — Evaluation, Position Reading, Watch

## Status

Accepted

## Context

The web SPA needs a domain model above the bridge's raw session concept to support a watchmaker's workflow: measuring a watch in multiple positions, comparing results, and tracking measurements over time. ADR 0003 defers the backend; initially this data lives in the browser (localStorage), but the shape is designed for eventual migration to a backend Postgres database.

Three genuine alternatives were considered:

- **No intermediate model** — map the bridge session 1:1 in the UI. Rejected because a single watch measurement spans multiple bridge sessions (one per position), and comparing/aggregating across them requires grouping.
- **Flat "Measurement" entity** — a single entity that tries to hold both evaluation-level and per-position data. Rejected because it conflates two scopes of data with different lifetimes and cardinalities.
- **Watch → Evaluation → Position Reading** (chosen) — three distinct entities with a clear hierarchy, each with its own lifecycle.

## Decision

The web UI's data model has three entities:

1. **Watch** — a physical watch movement being tracked. Lightweight: name, optional notes, created date. One-to-many with Evaluation. Survives across Evaluation cycles.
2. **Evaluation** — a single assessment of a Watch, consisting of one or more Position Readings. States: `draft` → `in_progress` → `complete`. Aggregates multi-position statistics (avg rate, max positional error) at completion.
3. **Position Reading** — one bridge session tagged with a position (dial up, dial down, crown left/right/down, crown up). Stores per-position statistics derived from bridge aggregate data (instant/short/long rate, beat error, amplitude). Created by pre-selecting a position then starting/stopping the bridge.

For MVP the model is implemented as TypeScript types in the web app and persisted to localStorage. When the backend is built (see ADR 0003), these types will become the Postgres schema.

## Consequences

- The bridge never needs to know about Evaluations or Positions — it remains a simple capture → stream → aggregate pipeline.
- Position Readings reference a bridge session ID, forming an audit trail back to raw tick data.
- The explicit state machine on Evaluation (`draft` → `in_progress` → `complete`) provides a clear trigger for computing multi-position aggregates and prevents overwriting completed data.
- TypeScript types serve as the single source of truth until backend migration; when that happens, the types become the basis for an OpenAPI spec or shared schema package.