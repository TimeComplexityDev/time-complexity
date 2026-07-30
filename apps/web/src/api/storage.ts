import type { Watch, Evaluation, PositionReading } from '../types';

const KEYS = {
  watches: 'watches',
  evaluations: 'evaluations',
  positionReadings: 'position_readings',
  pairToken: 'pair_token',
  bridgePort: 'bridge_port',
} as const;

function read<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function write<T>(key: string, value: T): void {
  localStorage.setItem(key, JSON.stringify(value));
}

// Watches
export function getWatches(): Watch[] {
  return read<Watch[]>(KEYS.watches, []);
}

export function saveWatch(watch: Watch): void {
  const watches = getWatches();
  const idx = watches.findIndex((w) => w.id === watch.id);
  if (idx >= 0) {
    watches[idx] = watch;
  } else {
    watches.push(watch);
  }
  write(KEYS.watches, watches);
}

export function deleteWatch(id: string): void {
  const watches = getWatches().filter((w) => w.id !== id);
  write(KEYS.watches, watches);
  const evals = getEvaluations().filter((e) => e.watch_id !== id);
  write(KEYS.evaluations, evals);
  const evalIds = new Set(evals.map((e) => e.id));
  const readings = getPositionReadings().filter((r) => !evalIds.has(r.evaluation_id));
  write(KEYS.positionReadings, readings);
}

// Evaluations
export function getEvaluations(): Evaluation[] {
  return read<Evaluation[]>(KEYS.evaluations, []);
}

export function getEvaluationsForWatch(watchId: string): Evaluation[] {
  return getEvaluations().filter((e) => e.watch_id === watchId);
}

export function saveEvaluation(evaluation: Evaluation): void {
  const evaluations = getEvaluations();
  const idx = evaluations.findIndex((e) => e.id === evaluation.id);
  if (idx >= 0) {
    evaluations[idx] = evaluation;
  } else {
    evaluations.push(evaluation);
  }
  write(KEYS.evaluations, evaluations);
}

export function deleteEvaluation(id: string): void {
  const evaluations = getEvaluations().filter((e) => e.id !== id);
  write(KEYS.evaluations, evaluations);
  const readings = getPositionReadings().filter((r) => r.evaluation_id !== id);
  write(KEYS.positionReadings, readings);
}

// Position Readings
export function getPositionReadings(): PositionReading[] {
  return read<PositionReading[]>(KEYS.positionReadings, []);
}

export function getPositionReadingsForEvaluation(evaluationId: string): PositionReading[] {
  return getPositionReadings().filter((r) => r.evaluation_id === evaluationId);
}

export function savePositionReading(reading: PositionReading): void {
  const readings = getPositionReadings();
  const idx = readings.findIndex((r) => r.id === reading.id);
  if (idx >= 0) {
    readings[idx] = reading;
  } else {
    readings.push(reading);
  }
  write(KEYS.positionReadings, readings);
}

export function deletePositionReading(id: string): void {
  const readings = getPositionReadings().filter((r) => r.id !== id);
  write(KEYS.positionReadings, readings);
}

// Pairing
export function getPairToken(): string | null {
  return localStorage.getItem(KEYS.pairToken);
}

export function setPairToken(token: string): void {
  localStorage.setItem(KEYS.pairToken, token);
}

export function clearPairToken(): void {
  localStorage.removeItem(KEYS.pairToken);
}

// Bridge port
export function getBridgePort(): number {
  const raw = localStorage.getItem(KEYS.bridgePort);
  return raw ? parseInt(raw, 10) : 9001;
}

export function setBridgePort(port: number): void {
  localStorage.setItem(KEYS.bridgePort, String(port));
}