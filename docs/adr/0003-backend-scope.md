# ADR 0003: Backend scope — deferred to post-MVP

## Status

Accepted

## Context

The original design describes a FastAPI backend for inventory, scraping notifications, and session summary ingestion. The user wants the backend to exist in the monorepo and eventually support watch servicing records (acquisition cost/date/source, repair parts, repair time/logs/notes) paired with timegrapher data.

## Decision

- Defer backend implementation until after the local bridge and web UI MVP is complete.
- The local bridge will persist session summaries locally (SQLite) and expose an internal/exportable format, but will not POST to a remote backend until that service exists.
- When the backend is built, it will ingest the local bridge's session summary format and add watch inventory/repair tracking domains.

## Consequences

- MVP scope is narrower and finishable: local bridge + web UI.
- Data format is chosen with future backend ingestion in mind, avoiding costly format migrations.
- Backend and local bridge can be built and released independently.
