import type { PositionReading } from '../types';
import { getPositionName } from '../types';

interface Props {
  reading: PositionReading;
  isSelected: boolean;
  onSelect: (reading: PositionReading) => void;
  onRetry: (reading: PositionReading) => void;
}

const stateLabels: Record<string, string> = {
  recording: 'Recording...',
  complete: 'Complete',
  failed: 'Failed',
};

function formatRate(rate: number | null): string {
  if (rate === null) return '—';
  return `${rate >= 0 ? '+' : ''}${rate.toFixed(1)} s/d`;
}

function formatBeatError(be: number | null): string {
  if (be === null) return '—';
  return `${(be * 1000).toFixed(2)} ms`;
}

function formatAmplitude(amp: number | null): string {
  if (amp === null) return '—';
  return `${amp.toFixed(0)}°`;
}

export default function PositionReadingCard({ reading, isSelected, onSelect, onRetry }: Props) {
  return (
    <div
      className={`card position-card ${isSelected ? 'selected' : ''} ${reading.state}`}
      onClick={() => onSelect(reading)}
    >
      <div className="card-body">
        <h4>{getPositionName(reading.position)}</h4>
        <span className={`badge badge-${reading.state}`}>{stateLabels[reading.state]}</span>
        {reading.state === 'complete' && (
          <div className="stats">
            <div className="stat">
              <span className="stat-label">Rate</span>
              <span className="stat-value">{formatRate(reading.rate_spd)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Beat Error</span>
              <span className="stat-value">{formatBeatError(reading.beat_error_s)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Amplitude</span>
              <span className="stat-value">{formatAmplitude(reading.amplitude)}</span>
            </div>
          </div>
        )}
        {reading.state === 'failed' && (
          <button
            className="btn-secondary btn-sm"
            onClick={(e) => {
              e.stopPropagation();
              onRetry(reading);
            }}
          >
            Retry
          </button>
        )}
      </div>
    </div>
  );
}