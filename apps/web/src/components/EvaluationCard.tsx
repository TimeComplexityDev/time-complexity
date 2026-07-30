import type { Evaluation } from '../types';

interface Props {
  evaluation: Evaluation;
  positionCount: number;
  completedCount: number;
  onClick: (evaluation: Evaluation) => void;
  onDelete: (id: string) => void;
}

const stateLabels: Record<string, string> = {
  draft: 'Draft',
  in_progress: 'In Progress',
  complete: 'Complete',
};

export default function EvaluationCard({
  evaluation,
  positionCount,
  completedCount,
  onClick,
  onDelete,
}: Props) {
  return (
    <div className="card evaluation-card" onClick={() => onClick(evaluation)}>
      <div className="card-body">
        <h3>{evaluation.name}</h3>
        <span className={`badge badge-${evaluation.state}`}>{stateLabels[evaluation.state]}</span>
        <p className="muted">
          {completedCount}/{positionCount} positions complete
        </p>
      </div>
      <button
        className="btn-danger btn-sm"
        onClick={(e) => {
          e.stopPropagation();
          onDelete(evaluation.id);
        }}
      >
        Delete
      </button>
    </div>
  );
}