export const POSITIONS = [
  'dial up',
  'dial down',
  'crown down',
  'crown left',
  'crown right',
] as const;

export type Position = (typeof POSITIONS)[number];

export type EvaluationState = 'draft' | 'in_progress' | 'complete';

export type PositionReadingState = 'recording' | 'complete' | 'failed';

export interface Watch {
  id: string;
  name: string;
  notes: string;
  created_at: string;
}

export interface Evaluation {
  id: string;
  watch_id: string;
  name: string;
  state: EvaluationState;
  bph: number | null;
  created_at: string;
}

export interface PositionReading {
  id: string;
  evaluation_id: string;
  position: Position;
  state: PositionReadingState;
  session_id: string | null;
  rate_spd: number | null;
  beat_error_s: number | null;
  amplitude: number | null;
  completed_at: string | null;
}

export interface TickEvent {
  type: 'tick';
  session_id: string;
  tick_index: number;
  timestamp: number;
  interval_s: number;
  rate_spd: number;
  amplitude: number;
}

export interface AggregateUpdate {
  type: 'aggregate';
  session_id: string;
  time: string;
  instant_rate_spd: number;
  short_avg_spd: number;
  long_ewma_spd: number;
  beat_error_s: number;
  amplitude: number;
}

export type StreamMessage = TickEvent | AggregateUpdate;

export function computeDefaultEvaluationName(watchName: string): string {
  const now = new Date();
  const date = now.toISOString().slice(0, 10);
  return `${watchName} — ${date}`;
}

export function allPositionsComplete(readings: PositionReading[]): boolean {
  const unique = new Set(readings.filter((r) => r.state === 'complete').map((r) => r.position));
  return unique.size === POSITIONS.length;
}

export function getCompletedPositions(readings: PositionReading[]): Position[] {
  return readings.filter((r) => r.state === 'complete').map((r) => r.position);
}

export function getPositionName(position: Position): string {
  const names: Record<Position, string> = {
    'dial up': 'Dial Up',
    'dial down': 'Dial Down',
    'crown down': 'Crown Down',
    'crown left': 'Crown Left',
    'crown right': 'Crown Right',
  };
  return names[position];
}