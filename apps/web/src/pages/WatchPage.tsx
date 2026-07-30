import { useState } from 'react';
import { useData } from '../context/DataContext';
import EvaluationCard from '../components/EvaluationCard';
import { getPositionReadingsForEvaluation } from '../api/storage';
import type { Evaluation } from '../types';

interface Props {
  watchId: string;
  onNavigate: (page: string, params?: Record<string, string>) => void;
}

export default function WatchPage({ watchId, onNavigate }: Props) {
  const { watches, evaluations, addEvaluation, removeEvaluation, refresh } = useData();
  const [showAdd, setShowAdd] = useState(false);
  const [newName, setNewName] = useState('');

  const watch = watches.find((w) => w.id === watchId);
  const watchEvaluations = evaluations.filter((e) => e.watch_id === watchId);

  if (!watch) {
    return (
      <div className="page">
        <p>Watch not found.</p>
        <button className="btn-secondary" onClick={() => onNavigate('home')}>
          Back
        </button>
      </div>
    );
  }

  const handleAdd = () => {
    const ev = addEvaluation(watchId, newName.trim() || undefined);
    setNewName('');
    setShowAdd(false);
    refresh();
    onNavigate('evaluation', { evaluationId: ev.id });
  };

  return (
    <div className="page">
      <header className="page-header">
        <button className="btn-secondary" onClick={() => onNavigate('home')}>
          &larr; Back
        </button>
        <h1>{watch.name}</h1>
        {watch.notes && <p className="muted">{watch.notes}</p>}
        <button className="btn-primary" onClick={() => setShowAdd(true)}>
          + New Evaluation
        </button>
      </header>

      {showAdd && (
        <div className="modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>New Evaluation for {watch.name}</h2>
            <input
              type="text"
              placeholder="Evaluation name (optional)"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              autoFocus
            />
            <div className="modal-actions">
              <button className="btn-secondary" onClick={() => setShowAdd(false)}>
                Cancel
              </button>
              <button className="btn-primary" onClick={handleAdd}>
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="card-grid">
        {watchEvaluations.length === 0 ? (
          <p className="muted">No evaluations yet. Click "+ New Evaluation" to start one.</p>
        ) : (
          watchEvaluations.map((ev: Evaluation) => {
            const readings = getPositionReadingsForEvaluation(ev.id);
            return (
              <EvaluationCard
                key={ev.id}
                evaluation={ev}
                positionCount={5}
                completedCount={readings.filter((r) => r.state === 'complete').length}
                onClick={() => onNavigate('evaluation', { evaluationId: ev.id })}
                onDelete={removeEvaluation}
              />
            );
          })
        )}
      </div>
    </div>
  );
}