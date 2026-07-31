import { useState, useEffect } from 'react';
import { setSource, listDevices } from '../api/bridge';

type SourceType = 'mic' | 'file';

interface SourceState {
  sourceType: SourceType;
  deviceName: string;
  filePath: string;
  loopPlayback: boolean;
}

interface Props {
  disabled?: boolean;
}

export default function SourceSelector({ disabled }: Props) {
  const [state, setState] = useState<SourceState>({
    sourceType: 'mic',
    deviceName: '',
    filePath: '',
    loopPlayback: false,
  });
  const [devices, setDevices] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    listDevices()
      .then(setDevices)
      .catch(() => setDevices([]));
  }, []);

  const handleApply = async () => {
    setSaving(true);
    setMessage(null);
    try {
      if (state.sourceType === 'mic') {
        await setSource({ type: 'mic', device_name: state.deviceName || undefined });
      } else {
        await setSource({ type: 'file', path: state.filePath, loop: state.loopPlayback });
      }
      setMessage('Source set');
    } catch (err) {
      setMessage(err instanceof Error ? err.message : 'Failed to set source');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="source-selector">
      <h4>Capture Source</h4>
      <div className="source-tabs">
        <button
          className={`source-tab ${state.sourceType === 'mic' ? 'active' : ''}`}
          onClick={() => setState((s) => ({ ...s, sourceType: 'mic' }))}
          disabled={disabled}
        >
          Microphone
        </button>
        <button
          className={`source-tab ${state.sourceType === 'file' ? 'active' : ''}`}
          onClick={() => setState((s) => ({ ...s, sourceType: 'file' }))}
          disabled={disabled}
        >
          Audio File
        </button>
      </div>

      <div className="source-fields">
        {state.sourceType === 'mic' ? (
          <div className="source-field">
            <label>Device</label>
            <select
              value={state.deviceName}
              onChange={(e) => setState((s) => ({ ...s, deviceName: e.target.value }))}
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
        ) : (
          <>
            <div className="source-field">
              <label>File path</label>
              <input
                type="text"
                placeholder="/path/to/recording.wav"
                value={state.filePath}
                onChange={(e) => setState((s) => ({ ...s, filePath: e.target.value }))}
                disabled={disabled}
              />
            </div>
            <label className="source-checkbox">
              <input
                type="checkbox"
                checked={state.loopPlayback}
                onChange={(e) => setState((s) => ({ ...s, loopPlayback: e.target.checked }))}
                disabled={disabled}
              />
              Loop playback
            </label>
          </>
        )}
      </div>

      <button className="btn-secondary btn-sm" onClick={handleApply} disabled={disabled || saving}>
        {saving ? 'Setting...' : 'Apply Source'}
      </button>

      {message && <span className={`source-message ${message === 'Source set' ? 'success' : 'error'}`}>{message}</span>}
    </div>
  );
}