import { getPairToken, getBridgePort } from './storage';
import type { StreamMessage } from '../types';

function baseUrl(): string {
  return `http://127.0.0.1:${getBridgePort()}`;
}

function authHeaders(): Record<string, string> {
  const token = getPairToken();
  if (!token) return {};
  return { Authorization: `Bearer ${token}` };
}

export async function pair(token: string): Promise<void> {
  const res = await fetch(`${baseUrl()}/pair`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token }),
  });
  if (!res.ok) throw new Error(`Pair failed: ${res.statusText}`);
}

export async function getStatus(): Promise<{
  running: boolean;
  session_id: string | null;
  bph: number;
}> {
  const res = await fetch(`${baseUrl()}/status`, { headers: authHeaders() });
  if (!res.ok) throw new Error(`Status failed: ${res.statusText}`);
  return res.json();
}

export async function startCapture(): Promise<{ status: string; session_id: string }> {
  const res = await fetch(`${baseUrl()}/start`, {
    method: 'POST',
    headers: authHeaders(),
  });
  if (!res.ok) throw new Error(`Start failed: ${res.statusText}`);
  return res.json();
}

export async function stopCapture(): Promise<{ status: string; session_id: string }> {
  const res = await fetch(`${baseUrl()}/stop`, {
    method: 'POST',
    headers: authHeaders(),
  });
  if (!res.ok) throw new Error(`Stop failed: ${res.statusText}`);
  return res.json();
}

export async function setParams(params: {
  bph?: number;
  bandpass_freq?: number;
  bandpass_q?: number;
}): Promise<void> {
  const res = await fetch(`${baseUrl()}/set_params`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(params),
  });
  if (!res.ok) throw new Error(`Set params failed: ${res.statusText}`);
}

export async function setSource(
  source: { type: 'mic'; device_name?: string } | { type: 'file'; path: string; loop?: boolean }
): Promise<void> {
  const res = await fetch(`${baseUrl()}/source`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(source),
  });
  if (!res.ok) throw new Error(`Set source failed: ${res.statusText}`);
}

export async function getSource(): Promise<{
  source: { type: 'mic'; device_name: string } | { type: 'file'; path: string; loop: boolean };
}> {
  const res = await fetch(`${baseUrl()}/source`, { headers: authHeaders() });
  if (!res.ok) throw new Error(`Get source failed: ${res.statusText}`);
  return res.json();
}

export async function listDevices(): Promise<string[]> {
  const res = await fetch(`${baseUrl()}/devices`, { headers: authHeaders() });
  if (!res.ok) throw new Error(`List devices failed: ${res.statusText}`);
  return res.json();
}

export function connectStream(
  onMessage: (msg: StreamMessage) => void,
  onError?: (err: Event) => void,
  onClose?: () => void
): WebSocket {
  const token = getPairToken();
  const ws = new WebSocket(`ws://127.0.0.1:${getBridgePort()}/stream?token=${token}`);

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data) as StreamMessage;
      onMessage(msg);
    } catch {
      // ignore malformed messages
    }
  };

  ws.onerror = (err) => onError?.(err);
  ws.onclose = () => onClose?.();

  return ws;
}