import { useEffect, useRef, useState, useCallback } from 'react';
import { useData } from '../context/DataContext';
import { usePairing } from '../context/PairingContext';
import { startCapture, stopCapture, setParams, getStatus, connectStream } from '../api/bridge';
import TickStream from '../components/TickStream';
import AggregateGauges from '../components/AggregateGauges';
import SourceSelector, { type SourceConfig } from '../components/SourceSelector';
import type { TickEvent, AggregateUpdate, StreamMessage, Navigate } from '../types';

interface Props {
  readingId: string;
  evaluationId: string;
  onNavigate: Navigate;
}

export default function CapturePage({ readingId, evaluationId, onNavigate }: Props) {
  const { positionReadings, evaluations, transitionEvaluation, markReadingComplete, markReadingFailed, setEvaluationBph } = useData();
  const { isPaired } = usePairing();

  const [ticks, setTicks] = useState<TickEvent[]>([]);
  const [aggregate, setAggregate] = useState<AggregateUpdate | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sourceConfig, setSourceConfig] = useState<SourceConfig>({ mic: {} });
  const wsRef = useRef<WebSocket | null>(null);
  const isRecordingRef = useRef(false);

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
    if (readingId) {
      markReadingFailed(readingId);
    }
    setIsRecording(false);
    isRecordingRef.current = false;
  }, [readingId, markReadingFailed]);

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
      await startCapture(sourceConfig);
      setIsRecording(true);
      isRecordingRef.current = true;

      if (evaluation && evaluation.state === 'draft') {
        transitionEvaluation(evaluationId, 'in_progress');
      }

      wsRef.current = connectStream(handleMessage, handleWsError, () => {
        if (isRecordingRef.current) {
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
        markReadingComplete(readingId, {
          rate_spd: aggregate.avg_rate_spd,
          beat_error_s: aggregate.beat_error_s,
          amplitude: aggregate.amplitude,
        });

        // Lock BPH on first reading
        if (evaluation && !evaluation.bph) {
          const status = await getStatus();
          setEvaluationBph(evaluationId, status.bph);
        }
      }

      setIsRecording(false);
      isRecordingRef.current = false;
      onNavigate({ name: 'evaluation', evaluationId });
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
        <button className="btn-secondary" onClick={() => onNavigate({ name: 'evaluation', evaluationId })}>
          Back
        </button>
      </div>
    );
  }

  return (
    <div className="page">
      <header className="page-header">
        <button className="btn-secondary" onClick={() => onNavigate({ name: 'evaluation', evaluationId })}>
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
        {!isRecording && <SourceSelector value={sourceConfig} onChange={setSourceConfig} disabled={isRecording} />}

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