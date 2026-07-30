import { POSITIONS, getPositionName } from '../types';
import type { Position, PositionReading } from '../types';

interface Props {
  readings: PositionReading[];
  selectedPosition: Position | null;
  onSelect: (position: Position) => void;
  disabled?: boolean;
}

function isPositionTaken(position: Position, readings: PositionReading[]): boolean {
  const completePositions = new Set(
    readings.filter((r) => r.state === 'complete').map((r) => r.position)
  );
  const recordingPositions = new Set(
    readings.filter((r) => r.state === 'recording').map((r) => r.position)
  );
  return completePositions.has(position) || recordingPositions.has(position);
}

export default function PositionSelector({ readings, selectedPosition, onSelect, disabled }: Props) {
  return (
    <div className="position-selector">
      <label>Position</label>
      <select
        value={selectedPosition ?? ''}
        onChange={(e) => onSelect(e.target.value as Position)}
        disabled={disabled}
      >
        <option value="" disabled>
          Select a position...
        </option>
        {POSITIONS.map((pos) => {
          const taken = isPositionTaken(pos, readings);
          return (
            <option key={pos} value={pos} disabled={taken}>
              {getPositionName(pos)} {taken ? '(done)' : ''}
            </option>
          );
        })}
      </select>
    </div>
  );
}