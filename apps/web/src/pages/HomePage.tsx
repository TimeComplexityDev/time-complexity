import { useState } from 'react';
import { useData } from '../context/DataContext';
import { usePairing } from '../context/PairingContext';
import WatchCard from '../components/WatchCard';
import type { Watch, Navigate } from '../types';

interface Props {
  onNavigate: Navigate;
}

export default function HomePage({ onNavigate }: Props) {
  const { watches, addWatch, removeWatch } = useData();
  const { isPaired, pair, isPairing, unpair } = usePairing();
  const [showAdd, setShowAdd] = useState(false);
  const [newName, setNewName] = useState('');
  const [newNotes, setNewNotes] = useState('');
  const [pairTokenInput, setPairTokenInput] = useState('');
  const [pairError, setPairError] = useState<string | null>(null);

  const handleAdd = () => {
    if (!newName.trim()) return;
    addWatch(newName.trim(), newNotes.trim());
    setNewName('');
    setNewNotes('');
    setShowAdd(false);
  };

  const handlePair = async () => {
    if (!pairTokenInput.trim()) return;
    setPairError(null);
    try {
      await pair(pairTokenInput.trim());
    } catch (err) {
      setPairError(err instanceof Error ? err.message : 'Pair failed');
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <h1>Time Complexity</h1>
        <div className="header-actions">
          {!isPaired ? (
            <div className="pair-form">
              <input
                type="text"
                placeholder="Pairing token"
                value={pairTokenInput}
                onChange={(e) => setPairTokenInput(e.target.value)}
              />
              <button className="btn-primary" onClick={handlePair} disabled={isPairing}>
                {isPairing ? 'Pairing...' : 'Pair'}
              </button>
            </div>
          ) : (
            <div className="paired-info">
              <span className="badge badge-paired">Paired</span>
              <button className="btn-secondary btn-sm" onClick={unpair}>
                Unpair
              </button>
            </div>
          )}
          {pairError && <span className="error-text">{pairError}</span>}
          <button className="btn-primary" onClick={() => setShowAdd(true)}>
            + New Watch
          </button>
        </div>
      </header>

      {showAdd && (
        <div className="modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>New Watch</h2>
            <input
              type="text"
              placeholder="Watch name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              autoFocus
            />
            <textarea
              placeholder="Notes (optional)"
              value={newNotes}
              onChange={(e) => setNewNotes(e.target.value)}
            />
            <div className="modal-actions">
              <button className="btn-secondary" onClick={() => setShowAdd(false)}>
                Cancel
              </button>
              <button className="btn-primary" onClick={handleAdd} disabled={!newName.trim()}>
                Add
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="card-grid">
        {watches.length === 0 ? (
          <p className="muted">No watches yet. Click "+ New Watch" to add one.</p>
        ) : (
          watches.map((w: Watch) => (
            <WatchCard
              key={w.id}
              watch={w}
              onClick={() => onNavigate({ name: 'watch', watchId: w.id })}
              onDelete={removeWatch}
            />
          ))
        )}
      </div>
    </div>
  );
}