import { useRef, useEffect } from 'react';
import type { TickEvent } from '../types';

interface Props {
  ticks: TickEvent[];
}

const WIDTH = 600;
const HEIGHT = 120;
const MAX_POINTS = 300;

export default function TickStream({ ticks }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, WIDTH, HEIGHT);

    const recent = ticks.slice(-MAX_POINTS);
    if (recent.length < 2) {
      ctx.fillStyle = '#666';
      ctx.font = '14px monospace';
      ctx.fillText('Waiting for ticks...', 16, HEIGHT / 2);
      return;
    }

    const rates = recent.map((t) => t.rate_spd);
    const minRate = Math.min(...rates);
    const maxRate = Math.max(...rates);
    const range = maxRate - minRate || 1;
    const midY = HEIGHT / 2;

    ctx.strokeStyle = '#4fc3f7';
    ctx.lineWidth = 1.5;
    ctx.beginPath();

    for (let i = 0; i < recent.length; i++) {
      const x = (i / recent.length) * WIDTH;
      const normalized = (rates[i] - minRate) / range;
      const y = midY - (normalized - 0.5) * HEIGHT * 0.8;
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();

    ctx.fillStyle = '#aaa';
    ctx.font = '11px monospace';
    ctx.fillText(`${recent[recent.length - 1].rate_spd.toFixed(1)} s/d`, 8, 16);
  }, [ticks]);

  return (
    <div className="tick-stream">
      <h4>Live Tick Stream</h4>
      <canvas ref={canvasRef} width={WIDTH} height={HEIGHT} className="stream-canvas" />
    </div>
  );
}