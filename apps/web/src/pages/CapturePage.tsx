import { useEffect, useRef, useState, useCallback } from 'react';
import { useData } from '../context/DataContext';
import { usePairing } from '../context/PairingContext';
import { startCapture, stopCapture, setParams, getStatus, connectStream } from '../api/bridge';
import TickStream from '../components/TickStream';
import AggregateGauges from '../components/AggregateGauges';
import SourceSelector from '../components/SourceSelector';
import type { TickEvent, AggregateUpdate, StreamMessage } from '../types';

interface Props {
  readingId: string;
  evaluationId: string;
  onNavigate: (page: string, params?: Record<string, string>) => void;
}

export default function CapturePage({ readingId, evaluationId, onNavigate }: Props) {
  const { positionReadings, updatePositionReading, evaluations, updateEvaluation } = useData();
  const { isPaired } = usePairing();

  const [ticks, setTicks] = useState<TickEvent[]>([]);
  const [aggregate, setAggregate] = useState<AggregateUpdate | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const reading = positionReadings.find((r) => r.id === readingId);
  const evaluation = evaluations.find((e) => e.id === evaluationId);

  const handleMessage = useCallback((msg: StreamMessage) => {
    if (msg.type === 'tick') {
      setTicks((prev) => [...prev.slice(-500), msg]);
    } else if (msg.type === 'aggregate') {
      setAggregate(msg);
    }
  }, []);

  const handleWsError = useCallback(() => {
    setError('WebSocket connection lost');
    if (reading && reading.state === 'recording') {
      reading.state = 'failed';
      updatePositionReading(reading);
    }
    setIsRecording(false);
  }, [reading, updatePositionReading]);

  const handleStart = async () => {
    if (!isPaired) {
      setError('Not paired with bridge. Go back and pair first.');
      return;
    }
    setError(null);
    setTicks([]);
    setAggregate(null);

    try {
      if (evaluation?.bph) {
        await setParams({ bph: evaluation.bph });
      }
      const result = await startCapture();
      setIsRecording(true);

      if (reading) {
        reading.session_id = result.session_id;
        reading.state = 'recording';
        updatePositionReading(reading);
      }

      if (evaluation && evaluation.state === 'draft') {
        evaluation.state = 'in_progress';
        updateEvaluation(evaluation);
      }

      wsRef.current = connectStream(handleMessage, handleWsError, () => {
        if (isRecording) {
          handleWsError();
        }
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start capture');
    }
  };

  const handleStop = async () => {
    try {
      await stopCapture();

      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }

      if (reading && aggregate) {
        reading.state = 'complete';
        reading.rate_spd = aggregate.long_ewma_spd;
        reading.beat_error_s = aggregate.beat_error_s;
        reading.amplitude = aggregate.amplitude;
        reading.completed_at = new Date().toISOString();
        updatePositionReading(reading);

        // Lock BPH on first reading
        if (evaluation && !evaluation.bph) {
          const status = await getStatus();
          evaluation.bph = status.bph;
          updateEvaluation(evaluation);
        }
      }

      setIsRecording(false);
      onNavigate('evaluation', { evaluationId });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to stop capture');
    }
  };

  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  if (!reading) {
    return (
      <div className="page">
        <p>Position reading not found.</p>
        <button className="btn-secondary" onClick={() => onNavigate('evaluation', { evaluationId })}>
          Back
        </button>
      </div>
    );
  }

  return (
    <div className="page">
      <header className="page-header">
        <button className="btn-secondary" onClick={() => onNavigate('evaluation', { evaluationId })}>
          &larr; Back
        </button>
        <h1>Capture: {reading.position}</h1>
        {evaluation?.bph && <p className="muted">BPH: {evaluation.bph}</p>}
      </header>

      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button className="btn-sm btn-secondary" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      )}

      <div className="capture-view">
        {!isRecording && <SourceSelector disabled={isRecording} />}

        <AggregateGauges aggregate={aggregate} />
        <TickStream ticks={ticks} />

        <div className="capture-actions">
          {!isRecording ? (
            <button className="btn-primary btn-large" onClick={handleStart} disabled={!isPaired}>
              Start Recording
            </button>
          ) : (
            <button className="btn-danger btn-large" onClick={handleStop}>
              Stop Recording
            </button>
          )}
        </div>
      </div>
    </div>
  );
}