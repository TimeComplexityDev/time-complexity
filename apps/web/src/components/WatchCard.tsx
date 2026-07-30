import type { Watch } from '../types';

interface Props {
  watch: Watch;
  onClick: (watch: Watch) => void;
  onDelete: (id: string) => void;
}

export default function WatchCard({ watch, onClick, onDelete }: Props) {
  return (
    <div className="card watch-card" onClick={() => onClick(watch)}>
      <div className="card-body">
        <h3>{watch.name}</h3>
        {watch.notes && <p className="muted">{watch.notes}</p>}
      </div>
      <button
        className="btn-danger btn-sm"
        onClick={(e) => {
          e.stopPropagation();
          onDelete(watch.id);
        }}
      >
        Delete
      </button>
    </div>
  );
}