# Web UI Refactor — Implementation Plan

## 1. `api/storage.ts` — Generic CRUD + port guard

- Extract a generic `upsert<T extends { id: string }>(key: string, item: T): void` that does `getAll` → `findIndex` → upsert-by-id → `write`.
- Extract a generic `list<T>(key: string): T[]` that wraps `read<T[]>(key, [])`.
- Rewrite `saveWatch`/`saveEvaluation`/`savePositionReading` as one-liners calling `upsert`.
- Rewrite `getWatches`/`getEvaluations`/`getPositionReadings` as one-liners calling `list`.
- Fix `getBridgePort`: guard against `NaN`/negative: `Number.isFinite(parsed) && parsed > 0 ? parsed : 9001`.

## 2. `context/DataContext.tsx` — Collapse to single refresh

- Replace per-entity setters with a single `refresh()` that calls `store.getWatches()`, `store.getEvaluations()`, `store.getPositionReadings()` and updates all three state arrays.
- Each mutation function (`addWatch`, `updateEvaluation`, etc.) calls `store.save*()` then `refresh()`.
- Unused `refresh()` is already exported from the context; verify it's used consistently.

## 3. State transitions → `DataContext` methods

- Add `transitionEvaluation(id: string, target: 'in_progress' | 'complete'): void` to `DataContext`. Validates legal transitions (`draft→in_progress`, `draft→complete`, `in_progress→complete`). Saves via `store.saveEvaluation` then calls `refresh()`.
- Add `markReadingComplete(id: string, stats: { rate_spd: number; beat_error_s: number; amplitude: number }): void` to `DataContext`. Sets `state='complete'`, `rate_spd`, `beat_error_s`, `amplitude`, `completed_at`. If all 5 positions now complete, auto-transitions the parent evaluation to `complete`.
- Add `markReadingFailed(id: string): void` to `DataContext`.
- Add `setEvaluationBph(id: string, bph: number): void` to `DataContext`.
- **`EvaluationPage.tsx`**: Replace all `evaluation.state = ...` + `updateEvaluation(evaluation)` with `transitionEvaluation(id, target)`.
- **`CapturePage.tsx`**: Replace all `reading.session_id = ...; reading.state = ...; updatePositionReading(reading)` with `markReadingComplete(id, stats)` and `markReadingFailed(id)`. Replace `evaluation.bph = ...; updateEvaluation(evaluation)` with `setEvaluationBph(id, bph)`.

## 4. `pages/EvaluationPage.tsx` — Extract duplicated filter

- Extract `const completeReadings = readings.filter(r => r.state === 'complete' && r.rate_spd !== null)` and reuse for avg rate and max positional error computation.

## 5. `types.ts` — Shared routing types

- Add `Page` discriminated union type and `Navigate` callback type:

```ts
export type Page =
  | { name: 'home' }
  | { name: 'watch'; watchId: string }
  | { name: 'evaluation'; evaluationId: string }
  | { name: 'capture'; readingId: string; evaluationId: string };

export type Navigate = (page: Page) => void;
```

- Update all `onNavigate` props across `App.tsx`, `HomePage`, `WatchPage`, `EvaluationPage`, `CapturePage` to use `Navigate` instead of `(string, Record<string,string>?)`.

## 6. `App.tsx` — Simplify routing

- Replace the `navigate` string-switch wrapper with `const navigate = (page: Page) => setPage(page)`.
- Update all call sites from `onNavigate('home')` to `onNavigate({ name: 'home' })`, `onNavigate('watch', { watchId })` to `onNavigate({ name: 'watch', watchId })`, etc.

## 7. `pages/WatchPage.tsx` — Use DataContext instead of direct storage

- Destructure `positionReadings` from `useData()` and filter locally instead of importing `getPositionReadingsForEvaluation` from `store`.

## 8. `components/SourceSelector.tsx` — Remove dead prop

- Remove `onSourceChange?: () => void` prop and its usage.

## 9. `types.ts` — Shared `formatRate` and `stateLabels`

- Export `formatRate(v: number | null): string` (handles null → `—`, positive → `+N.N s/d`, negative → `-N.N s/d`).
- Export `stateLabels: Record<string, string>` for evaluation and position reading states.
- Import and use in `AggregateGauges.tsx`, `TickStream.tsx`, `PositionReadingCard.tsx`, `EvaluationCard.tsx`.

## 10. BPH override UI

- Add a BPH dropdown to `EvaluationPage.tsx` header (visible when `state === 'in_progress'`).
- Options: 18000, 19800, 21600, 25200, 28800, 36000 (matches bridge `COMMON_BPH`).
- On change: call `POST /set_params { bph }` and `setEvaluationBph(id, bph)`.

## 11. `CapturePage.tsx` — Fix stale closure

- Add `isRecordingRef = useRef(false)`.
- Set `isRecordingRef.current = true` alongside `setIsRecording(true)`.
- Read `isRecordingRef.current` instead of `isRecording` in the WebSocket close callback.