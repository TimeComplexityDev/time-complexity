# ADR 0001: Monorepo layout with multi-context domain docs

## Status

Accepted

## Context

The Time Complexity repo will contain multiple independently releasable components: a web UI, an optional backend, and a local bridge binary. We need a directory layout that keeps concerns separated while allowing cross-component coordination. We also need domain docs to stay current as the codebase grows.

## Decision

- Use a monorepo under `apps/` with three top-level contexts: `web`, `backend`, `local-bridge`.
- Record a root `CONTEXT-MAP.md` that points to per-component `CONTEXT.md` files.
- Do not maintain a shared root `CONTEXT.md`; each component's context is authoritative for itself.
- Use local markdown (`.scratch/<feature>/`) for issue tracking because this is a solo project without a remote issue tracker workflow.
- Persist architectural decisions as ADRs under `docs/adr/`.

## Consequences

- Components can be developed and released independently.
- Domain vocabulary is kept close to the code that uses it.
- Issue tracking is lightweight and offline-first.
- ADRs provide a lightweight record of architectural choices.
