# Coding Standards

## State transition discipline

Do not mutate React state objects in-place. State transitions (e.g., `draft → in_progress → complete`) are modelled on the entity, not performed by direct property assignment in a component. Each entity gets explicit transition methods on its context provider (`transitionEvaluation`, `markReadingComplete`, etc.). Components call these methods; they do not write to `.state`, `.bph`, or other fields directly.

*Why:* In-place mutation couples the component to the entity's internal shape, makes transitions hard to test, and breaks React's immutable-edit assumptions.

## Data access layering

Components read and write data through the context layer. Do not import `store` (localStorage) or `bridge` (REST) directly inside page/component files — the context exists to mediate those dependencies.

*Why:* Direct storage imports bypass re-render triggers and create scattered dependencies that are hard to replace when the backend arrives.

## Shared type extraction

When a format function (`formatRate`, `formatBeatError`), label map (`EVALUATION_STATE_LABELS`), or routing shape appears in more than one file, extract it to `types.ts` immediately. Do not wait for a third occurrence.

*Why:* Duplicated formatting logic drifts over time and creates inconsistent output. A single source of truth eliminates the drift.

## Generic CRUD helpers

Use generic `upsert<T extends { id: string }>(key, item)` and `list<T>(key)` helpers instead of copy-pasting the same `findIndex`/`push-or-replace`/`write` pattern per entity.

*Why:* Per-entity copy-paste multiplies bugs — every duplicated block is a place to forget to update when the pattern changes.

## Closure freshness with refs

When a callback (WebSocket handler, timeout, event listener) needs to read the *current* value of a state variable at invocation time, store that value in a `useRef` alongside the `useState`. Pass the ref into the callback, not the state variable.

*Why:* Closures capture state at render time. If the component re-renders before the callback fires, the captured value is stale. Refs don't have this problem.

## Routing type safety

Page routes use a discriminated union type (`Page`) with exact parameter requirements per page. Navigation functions accept a `Page` object, never a loose `(string, Record<string, string>?)` pair. The render switch enumerates all cases exhaustively.

*Why:* String-based routing makes invalid page names and missing parameters a runtime concern. A discriminated union makes them a compile-time error.

## Smell baseline (from Fowler, *Refactoring* ch.3)

All code should be reviewed against these smells. A smell is a judgement call, never a hard violation — but when one is spotted, consider whether the fix is cheap enough to do now:

- **Mysterious Name** — a function or type whose name doesn't reveal what it does
- **Duplicated Code** — the same logic shape in more than one place
- **Feature Envy** — a method that reaches into another object's data more than its own
- **Data Clumps** — the same fields or params travelling together, wanting a type
- **Primitive Obsession** — a primitive standing in for a domain concept
- **Repeated Switches** — the same `switch`/`if`-cascade on the same type across files
- **Shotgun Surgery** — one logical change touching many files
- **Divergent Change** — one file changing for several unrelated reasons
- **Speculative Generality** — abstraction added for needs not yet required
- **Message Chains** — long `a.b().c().d()` navigation
- **Middle Man** — a class that mostly delegates onward
- **Refused Bequest** — a subclass that ignores most of what it inherits