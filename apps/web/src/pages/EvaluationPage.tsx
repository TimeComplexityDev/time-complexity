import { useState } from 'react';
import { useData } from '../context/DataContext';
import { setParams } from '../api/bridge';
import PositionReadingCard from '../components/PositionReadingCard';
import PositionSelector from '../components/PositionSelector';
import { allPositionsComplete, COMMON_BPH, EVALUATION_STATE_LABELS, formatRate } from '../types';
import type { Position, PositionReading, Navigate } from '../types';

interface Props {
  evaluationId: string;
  onNavigate: Navigate;
}

export default function EvaluationPage({ evaluationId, onNavigate }: Props) {
  const { evaluations, positionReadings, addPositionReading, updateEvaluation, transitionEvaluation, setEvaluationBph } = useData();
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [editingName, setEditingName] = useState(false);
  const [nameInput, setNameInput] = useState('');

  const evaluation = evaluations.find((e) => e.id === evaluationId);
  const readings = positionReadings.filter((r) => r.evaluation_id === evaluationId);

  if (!evaluation) {
    return (
      <div className="page">
        <p>Evaluation not found.</p>
        <button className="btn-secondary" onClick={() => onNavigate({ name: 'home' })}>
          Back
        </button>
      </div>
    );
  }

  const completeReadings = readings.filter((r) => r.state === 'complete' && r.rate_spd !== null);
  const completedCount = readings.filter((r) => r.state === 'complete').length;
  const isComplete = evaluation.state === 'complete';

  const handleStartCapture = () => {
    if (!selectedPosition) return;

    if (evaluation.state === 'draft') {
      transitionEvaluation(evaluationId, 'in_progress');
    }

    const reading = addPositionReading(evaluationId, selectedPosition);
    onNavigate({ name: 'capture', readingId: reading.id, evaluationId });
  };

  const handleRetry = (reading: PositionReading) => {
    const newReading = addPositionReading(evaluationId, reading.position);
    onNavigate({ name: 'capture', readingId: newReading.id, evaluationId });
  };

  const handleFinish = () => {
    if (evaluation.state === 'in_progress' && completedCount > 0) {
      transitionEvaluation(evaluationId, 'complete');
    }
  };

  const handleSaveName = () => {
    if (nameInput.trim()) {
      evaluation.name = nameInput.trim();
      updateEvaluation(evaluation);
    }
    setEditingName(false);
  };

  const handleBphChange = async (bph: number) => {
    setEvaluationBph(evaluationId, bph);
    try {
      await setParams({ bph });
    } catch {
      // ignore bridge errors
    }
  };

  const canFinish = evaluation.state === 'in_progress' && completedCount > 0 && !allPositionsComplete(readings);

  const avgRate = completeReadings.length > 0
    ? completeReadings.reduce((sum, r) => sum + (r.rate_spd as number), 0) / completeReadings.length
    : null;

  const completeRates = completeReadings.map((r) => r.rate_spd as number);
  const maxPosError = completeRates.length >= 2 ? Math.max(...completeRates) - Math.min(...completeRates) : null;

  return (
    <div className="page">
      <header className="page-header">
        <button className="btn-secondary" onClick={() => onNavigate({ name: 'watch', watchId: evaluation.watch_id })}>
          &larr; Back
        </button>
        {editingName ? (
          <div className="inline-edit">
            <input
              type="text"
              value={nameInput}
              onChange={(e) => setNameInput(e.target.value)}
              onBlur={handleSaveName}
              onKeyDown={(e) => e.key === 'Enter' && handleSaveName()}
              autoFocus
            />
          </div>
        ) : (
          <h1
            className="editable-title"
            onClick={() => {
              setNameInput(evaluation.name);
              setEditingName(true);
            }}
          >
            {evaluation.name}
          </h1>
        )}
        <span className={`badge badge-${evaluation.state}`}>
          {EVALUATION_STATE_LABELS[evaluation.state]}
        </span>
        <p className="muted">
          {completedCount}/5 positions completed
        </p>
        {evaluation.state !== 'draft' && (
          <div className="bph-control">
            <label>BPH</label>
            <select
              value={evaluation.bph ?? 28800}
              onChange={(e) => handleBphChange(Number(e.target.value))}
              disabled={isComplete}
            >
              {COMMON_BPH.map((bph) => (
                <option key={bph} value={bph}>
                  {bph}
                </option>
              ))}
            </select>
          </div>
        )}
      </header>

      {evaluation.state === 'complete' && (
        <div className="summary">
          <h3>Summary</h3>
          {avgRate !== null && (
            <div className="summary-stat">
              <span className="stat-label">Average Rate</span>
              <span className="stat-value">{formatRate(avgRate)}</span>
            </div>
          )}
          {maxPosError !== null && (
            <div className="summary-stat">
              <span className="stat-label">Max Positional Error</span>
              <span className="stat-value">{maxPosError.toFixed(1)} s/d</span>
            </div>
          )}
        </div>
      )}

      {!isComplete && (
        <div className="capture-controls">
          <PositionSelector
            readings={readings}
            selectedPosition={selectedPosition}
            onSelect={setSelectedPosition}
          />
          <button
            className="btn-primary"
            onClick={handleStartCapture}
            disabled={!selectedPosition}
          >
            Start Capture
          </button>
          {canFinish && (
            <button className="btn-secondary" onClick={handleFinish}>
              Finish Early
            </button>
          )}
        </div>
      )}

      <div className="card-grid">
        {readings.length === 0 ? (
          <p className="muted">No position readings yet. Select a position and start capture.</p>
        ) : (
          readings.map((r) => (
            <PositionReadingCard
              key={r.id}
              reading={r}
              isSelected={false}
              onSelect={() => {
                if (r.state === 'recording' || r.state === 'failed') {
                  onNavigate({ name: 'capture', readingId: r.id, evaluationId });
                }
              }}
              onRetry={handleRetry}
            />
          ))
        )}
      </div>
    </div>
  );
}