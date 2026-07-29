# Domain docs

This repo uses a **multi-context** layout.

## Context map

- `apps/web/CONTEXT.md` — Browser UI (React SPA)
- `apps/backend/CONTEXT.md` — Optional FastAPI backend
- `apps/local-bridge/CONTEXT.md` — Local macOS bridge (Rust audio/DSP binary)

## How to read

- Start at `CONTEXT-MAP.md` for a map of all components.
- Read a component's `CONTEXT.md` before editing that component's code.
- Treat each component's `CONTEXT.md` as authoritative for its own domain; the root `CONTEXT.md` is not used.

## ADRs

Architectural decisions are recorded as Markdown files under `docs/adr/`.

## Why multi-context

The components are independently releasable and use different languages/frameworks. A shared root `CONTEXT.md` would mix domains and become stale. A context map keeps each component's vocabulary local.
