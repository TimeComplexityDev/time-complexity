import type { AggregateUpdate } from '../types';

interface Props {
  aggregate: AggregateUpdate | null;
}

function formatRate(v: number): string {
  return `${v >= 0 ? '+' : ''}${v.toFixed(1)}`;
}

function formatBeatError(v: number): string {
  return `${(v * 1000).toFixed(2)} ms`;
}

export default function AggregateGauges({ aggregate }: Props) {
  if (!aggregate) {
    return (
      <div className="aggregate-gauges">
        <h4>Aggregate Metrics</h4>
        <p className="muted">Waiting for data...</p>
      </div>
    );
  }

  return (
    <div className="aggregate-gauges">
      <h4>Aggregate Metrics</h4>
      <div className="gauge-grid">
        <div className="gauge">
          <span className="gauge-label">Instant Rate</span>
          <span className="gauge-value">{formatRate(aggregate.instant_rate_spd)}</span>
          <span className="gauge-unit">s/d</span>
        </div>
        <div className="gauge">
          <span className="gauge-label">Short Avg</span>
          <span className="gauge-value">{formatRate(aggregate.short_avg_spd)}</span>
          <span className="gauge-unit">s/d</span>
        </div>
        <div className="gauge">
          <span className="gauge-label">Long EWMA</span>
          <span className="gauge-value">{formatRate(aggregate.long_ewma_spd)}</span>
          <span className="gauge-unit">s/d</span>
        </div>
        <div className="gauge">
          <span className="gauge-label">Beat Error</span>
          <span className="gauge-value">{formatBeatError(aggregate.beat_error_s)}</span>
        </div>
        <div className="gauge">
          <span className="gauge-label">Amplitude</span>
          <span className="gauge-value">{aggregate.amplitude.toFixed(0)}°</span>
        </div>
      </div>
    </div>
  );
}