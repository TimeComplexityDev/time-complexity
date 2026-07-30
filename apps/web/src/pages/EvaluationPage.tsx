import { useState } from 'react';
import { useData } from '../context/DataContext';
import PositionReadingCard from '../components/PositionReadingCard';
import PositionSelector from '../components/PositionSelector';
import { allPositionsComplete } from '../types';
import type { Position, PositionReading } from '../types';

interface Props {
  evaluationId: string;
  onNavigate: (page: string, params?: Record<string, string>) => void;
}

export default function EvaluationPage({ evaluationId, onNavigate }: Props) {
  const { evaluations, positionReadings, updateEvaluation, addPositionReading } = useData();
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);

  const evaluation = evaluations.find((e) => e.id === evaluationId);
  const readings = positionReadings.filter((r) => r.evaluation_id === evaluationId);
  const [editingName, setEditingName] = useState(false);
  const [nameInput, setNameInput] = useState('');

  if (!evaluation) {
    return (
      <div className="page">
        <p>Evaluation not found.</p>
        <button className="btn-secondary" onClick={() => onNavigate('home')}>
          Back
        </button>
      </div>
    );
  }

  const completedCount = readings.filter((r) => r.state === 'complete').length;
  const isComplete = evaluation.state === 'complete';

  const handleStartCapture = () => {
    if (!selectedPosition) return;

    if (evaluation.state === 'draft') {
      evaluation.state = 'in_progress';
      updateEvaluation(evaluation);
    }

    const reading = addPositionReading(evaluationId, selectedPosition);
    onNavigate('capture', { readingId: reading.id, evaluationId });
  };

  const handleRetry = (reading: PositionReading) => {
    const newReading = addPositionReading(evaluationId, reading.position);
    onNavigate('capture', { readingId: newReading.id, evaluationId });
  };

  const handleFinish = () => {
    if (evaluation.state === 'in_progress' && completedCount > 0) {
      evaluation.state = 'complete';
      updateEvaluation(evaluation);
    }
  };

  const handleSaveName = () => {
    if (nameInput.trim()) {
      evaluation.name = nameInput.trim();
      updateEvaluation(evaluation);
    }
    setEditingName(false);
  };

  const canFinish = evaluation.state === 'in_progress' && completedCount > 0 && !allPositionsComplete(readings);
  const autoCompleteReached = allPositionsComplete(readings) && evaluation.state === 'in_progress';

  if (autoCompleteReached && !isComplete) {
    evaluation.state = 'complete';
    updateEvaluation(evaluation);
  }

  const avgRate =
    completedCount > 0
      ? readings
          .filter((r) => r.state === 'complete' && r.rate_spd !== null)
          .reduce((sum, r) => sum + (r.rate_spd ?? 0), 0) / completedCount
      : null;

  const completeRates = readings
    .filter((r) => r.state === 'complete' && r.rate_spd !== null)
    .map((r) => r.rate_spd as number);
  const maxPosError = completeRates.length >= 2 ? Math.max(...completeRates) - Math.min(...completeRates) : null;

  return (
    <div className="page">
      <header className="page-header">
        <button className="btn-secondary" onClick={() => onNavigate('watch', { watchId: evaluation.watch_id })}>
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
          {evaluation.state === 'draft' ? 'Draft' : evaluation.state === 'in_progress' ? 'In Progress' : 'Complete'}
        </span>
        <p className="muted">
          {completedCount}/5 positions completed
        </p>
        {evaluation.bph && <p className="muted">BPH: {evaluation.bph}</p>}
      </header>

      {evaluation.state === 'complete' && (
        <div className="summary">
          <h3>Summary</h3>
          {avgRate !== null && (
            <div className="summary-stat">
              <span className="stat-label">Average Rate</span>
              <span className="stat-value">{avgRate >= 0 ? '+' : ''}{avgRate.toFixed(1)} s/d</span>
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
                  onNavigate('capture', { readingId: r.id, evaluationId });
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