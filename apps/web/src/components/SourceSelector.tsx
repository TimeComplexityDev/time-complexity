import { useState, useEffect } from 'react';
import { listDevices } from '../api/bridge';

export type SourceConfig =
  | { mic: { device_name?: string } }
  | { file: { path: string; loop_playback?: boolean } }
  | { simulator: { bph: number; drift_s_per_day: number; beat_error_ms: number } };

type SourceType = 'mic' | 'file' | 'simulator';

interface Props {
  value: SourceConfig;
  onChange: (config: SourceConfig) => void;
  disabled?: boolean;
}

export default function SourceSelector({ value, onChange, disabled }: Props) {
  const [devices, setDevices] = useState<string[]>([]);

  const sourceType: SourceType = 'mic' in value ? 'mic' : 'file' in value ? 'file' : 'simulator';

  useEffect(() => {
    listDevices()
      .then(setDevices)
      .catch(() => setDevices([]));
  }, []);

  return (
    <div className="source-selector">
      <h4>Capture Source</h4>
      <div className="source-tabs">
        <button
          className={`source-tab ${sourceType === 'mic' ? 'active' : ''}`}
          onClick={() => onChange({ mic: {} })}
          disabled={disabled}
        >
          Microphone
        </button>
        <button
          className={`source-tab ${sourceType === 'file' ? 'active' : ''}`}
          onClick={() => onChange({ file: { path: '', loop_playback: false } })}
          disabled={disabled}
        >
          Audio File
        </button>
        <button
          className={`source-tab ${sourceType === 'simulator' ? 'active' : ''}`}
          onClick={() => onChange({ simulator: { bph: 21600, drift_s_per_day: 0, beat_error_ms: 0 } })}
          disabled={disabled}
        >
          Simulator
        </button>
      </div>

      <div className="source-fields">
        {sourceType === 'mic' && (
          <div className="source-field">
            <label>Device</label>
            <select
              value={'mic' in value ? value.mic.device_name ?? '' : ''}
              onChange={(e) => onChange({ mic: { device_name: e.target.value || undefined } })}
              disabled={disabled}
            >
              <option value="">Default device</option>
              {devices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </div>
        )}

        {sourceType === 'file' && 'file' in value && (
          <>
            <div className="source-field">
              <label>File path</label>
              <input
                type="text"
                placeholder="/path/to/recording.wav"
                value={value.file.path}
                onChange={(e) => onChange({ file: { ...value.file, path: e.target.value } })}
                disabled={disabled}
              />
            </div>
            <label className="source-checkbox">
              <input
                type="checkbox"
                checked={value.file.loop_playback ?? false}
                onChange={(e) => onChange({ file: { ...value.file, loop_playback: e.target.checked } })}
                disabled={disabled}
              />
              Loop playback
            </label>
          </>
        )}

        {sourceType === 'simulator' && 'simulator' in value && (
          <>
            <div className="source-field">
              <label>BPH</label>
              <input
                type="number"
                min={3600}
                max={72000}
                step={1800}
                value={value.simulator.bph}
                onChange={(e) => onChange({ simulator: { ...value.simulator, bph: parseInt(e.target.value, 10) || 21600 } })}
                disabled={disabled}
              />
            </div>
            <div className="source-field">
              <label>Drift (s/day)</label>
              <input
                type="number"
                step={1}
                value={value.simulator.drift_s_per_day}
                onChange={(e) => onChange({ simulator: { ...value.simulator, drift_s_per_day: parseFloat(e.target.value) || 0 } })}
                disabled={disabled}
              />
            </div>
            <div className="source-field">
              <label>Beat error (ms)</label>
              <input
                type="number"
                step={0.1}
                min={0}
                value={value.simulator.beat_error_ms}
                onChange={(e) => onChange({ simulator: { ...value.simulator, beat_error_ms: parseFloat(e.target.value) || 0 } })}
                disabled={disabled}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}