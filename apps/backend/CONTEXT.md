# Context: apps/backend

This directory contains the FastAPI backend for the watchmaker's inventory and timegrapher measurement platform.

## Technology

- FastAPI (Python)
- SQLAlchemy 2.0 ORM style
- Alembic for migrations
- SQLite for local development (`./data/dev.db`, gitignored)
- PostgreSQL for production (e.g. Supabase)
- Bearer API token auth (from `AUTH_TOKEN` in `.env`)

## Domain vocabulary

### Core entities

- **Watch Make**: A watch brand/company (e.g. Rolex, Seiko, Omega). Has a name and optional parent company (e.g. Swatch Group).
- **Movement Type**: A specific caliber/movement reference (e.g. ETA 2824-2, Calibre 3135). Has a name, type (mechanical_automatic, mechanical_manual, quartz, spring_drive), jewel count, and BPH.
- **Watch Model**: A documented model reference (e.g. Rolex Submariner 124060). Links a Watch Make to a Movement Type. Has a name and optional URL.
- **Watch**: A physical watch being tracked. May reference a Watch Model (optional — you can track a watch without knowing its model). Has optional serial number, year of production (text field, can be a range), acquisition details (purchase date, source, price in CHF), condition status (running/broken), and free-text status notes.

### Possession

- **Possession Event**: Records a watch coming into or leaving the user's physical possession. Each event has a direction (in/out), context (owned — the user owns this watch; customer — the watch belongs to a client), a free-text reason (e.g. purchase, sale, service, loan), event date (defaults to today), and optional notes. The watch's current state is derived from its latest event: current ownership = latest event's context, current possession = (direction == "in").

### Measurements

- **Evaluation**: A measurement session for a Watch, consisting of one or more Position Readings. State machine: `in_progress` → `complete` (no draft state, no going back). On completion, pre-computed aggregates are stored: average rate, maximum positional delta (positional error), average beat error, average amplitude.
- **Position Reading**: A single measurement of a Watch in one of the six ISO 3159 positions (dial_up, dial_down, crown_up, crown_down, crown_left, crown_right). Stores rate (s/day), beat error (ms), amplitude (degrees), and timestamp.

## API design

Standard CRUD per entity via a generic router factory. Endpoints use Bearer token auth. Entities are exposed as:

- `GET/POST /api/makes` and `GET/PUT/DELETE /api/makes/{id}`
- Same pattern for `/movements`, `/models`, `/watches`, `/possession-events`, `/evaluations`, `/position-readings`
- Position Readings are nested under evaluations: `/evaluations/{id}/position-readings`

## Current state

Active development. The backend replaces the earlier frontend-only data model (ADR 0008) and the deferred-backend posture (ADR 0003). The local bridge no longer stores session data — raw measurements are ephemeral, only the summary data in Position Readings persists.