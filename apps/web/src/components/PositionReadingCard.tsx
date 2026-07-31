import type { PositionReading } from '../types';
import { getPositionName, READING_STATE_LABELS, formatRate, formatBeatError, formatAmplitude } from '../types';

interface Props {
  reading: PositionReading;
  isSelected: boolean;
  onSelect: (reading: PositionReading) => void;
  onRetry: (reading: PositionReading) => void;
}

export default function PositionReadingCard({ reading, isSelected, onSelect, onRetry }: Props) {
  return (
    <div
      className={`card position-card ${isSelected ? 'selected' : ''} ${reading.state}`}
      onClick={() => onSelect(reading)}
    >
      <div className="card-body">
        <h4>{getPositionName(reading.position)}</h4>
        <span className={`badge badge-${reading.state}`}>{READING_STATE_LABELS[reading.state]}</span>
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