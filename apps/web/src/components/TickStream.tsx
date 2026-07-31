import { useRef, useEffect } from 'react';
import type { TickEvent } from '../types';

interface Props {
  ticks: TickEvent[];
}

const WIDTH = 600;
const HEIGHT = 160;
const MAX_POINTS = 300;
const MARGIN = { top: 28, right: 20, bottom: 28, left: 52 };

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
      ctx.font = '12px monospace';
      ctx.fillText('Waiting for ticks...', MARGIN.left + 8, HEIGHT / 2);
      return;
    }

    const plotW = WIDTH - MARGIN.left - MARGIN.right;
    const plotH = HEIGHT - MARGIN.top - MARGIN.bottom;

    const t0 = recent[0].timestamp;
    const tEnd = recent[recent.length - 1].timestamp;
    const timeSpan = tEnd - t0 || 1;

    const rates = recent.map((t) => t.rate_spd);
    const minRate = Math.min(0, ...rates);
    const maxRate = Math.max(0, ...rates);
    const range = maxRate - minRate || 1;
    const pad = range * 0.1;
    const yMin = minRate - pad;
    const yMax = maxRate + pad;
    const yRange = yMax - yMin;

    // grid & axes
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 0.5;
    ctx.fillStyle = '#888';
    ctx.font = '10px monospace';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';

    const yTicks = 4;
    for (let i = 0; i <= yTicks; i++) {
      const val = yMin + (yRange * i) / yTicks;
      const y = MARGIN.top + plotH - ((val - yMin) / yRange) * plotH;
      ctx.fillText(val.toFixed(1), MARGIN.left - 6, y);
      ctx.beginPath();
      ctx.moveTo(MARGIN.left, y);
      ctx.lineTo(WIDTH - MARGIN.right, y);
      ctx.stroke();
    }

    // zero line
    const zeroY = MARGIN.top + plotH - ((0 - yMin) / yRange) * plotH;
    ctx.strokeStyle = '#555';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(MARGIN.left, zeroY);
    ctx.lineTo(WIDTH - MARGIN.right, zeroY);
    ctx.stroke();

    // x-axis time labels
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.fillStyle = '#888';
    const xTicks = 4;
    for (let i = 0; i <= xTicks; i++) {
      const t = t0 + (timeSpan * i) / xTicks;
      const x = MARGIN.left + (plotW * i) / xTicks;
      ctx.fillText(`${(t - t0).toFixed(1)}s`, x, MARGIN.top + plotH + 6);
    }

    // border
    ctx.strokeStyle = '#444';
    ctx.lineWidth = 0.5;
    ctx.strokeRect(MARGIN.left, MARGIN.top, plotW, plotH);

    // data line
    ctx.strokeStyle = '#4fc3f7';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let i = 0; i < recent.length; i++) {
      const x = MARGIN.left + ((recent[i].timestamp - t0) / timeSpan) * plotW;
      const y = MARGIN.top + plotH - ((rates[i] - yMin) / yRange) * plotH;
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();

    // latest value label
    ctx.fillStyle = '#aaa';
    ctx.font = '11px monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'bottom';
    const lastRate = recent[recent.length - 1].rate_spd;
    ctx.fillText(`${lastRate >= 0 ? '+' : ''}${lastRate.toFixed(1)} s/d`, MARGIN.left + 4, MARGIN.top - 4);

    // y-axis label
    ctx.fillStyle = '#666';
    ctx.font = '9px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'bottom';
    ctx.save();
    ctx.translate(10, MARGIN.top + plotH / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText('rate (s/d)', 0, 0);
    ctx.restore();
  }, [ticks]);

  return (
    <div className="tick-stream">
      <h4>Live Tick Stream</h4>
      <canvas ref={canvasRef} width={WIDTH} height={HEIGHT} className="stream-canvas" />
    </div>
  );
}