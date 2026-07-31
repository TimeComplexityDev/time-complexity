# Agent Instructions

- This repo uses kilo for agent-driven development.
- Skills are located in `.agents/skills/` and are auto-discovered by kilo.
- Use the available skills to guide your work (e.g., `/implement`, `/tdd`, `/code-review`).
- Run typechecking and tests regularly. Commit your work to the current branch.

## Agent skills

### Issue tracker

Issues live as GitHub issues in this repo. Use the `gh` CLI for all operations. See `docs/agents/issue-tracker.md`.

### Triage labels

Default label vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context layout: root `CONTEXT-MAP.md` pointing to per-component `CONTEXT.md` files. See `docs/agents/domain.md`.

## Dependency policy

Prefer well-established, actively maintained libraries over hand-rolled implementations for standard tasks (date/time, serialization, audio decoding, HTTP, CLI parsing). Only hand-roll when the library adds unacceptable weight for the use case, and document the rationale inline.

## Coding standards

Review `CODING_STANDARDS.md` before writing new components or refactoring existing ones. The code-review skill checks against these rules plus the Fowler smell baseline documented there.