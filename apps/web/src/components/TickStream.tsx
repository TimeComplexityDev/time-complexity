import { useRef, useEffect } from 'react';
import type { TickEvent } from '../types';

interface Props {
  ticks: TickEvent[];
}

const WIDTH = 600;
const PANEL_HEIGHT = 100;
const PANEL_GAP = 12;
const MAX_POINTS = 300;
const MARGIN = { top: 22, right: 20, bottom: 22, left: 56 };
const TOTAL_HEIGHT = PANEL_HEIGHT * 3 + PANEL_GAP * 2;

const COLORS = {
  tic: '#4fc3f7',
  tok: '#ffb74d',
  amplitude: '#81c784',
  beatError: '#ef5350',
  grid: '#333',
  zeroLine: '#555',
  axisText: '#888',
  border: '#444',
};

interface PanelConfig {
  title: string;
  unit: string;
  color: string;
  yMin: number;
  yMax: number;
  baseline?: number;
}

function drawPanel(
  ctx: CanvasRenderingContext2D,
  panelY: number,
  cfg: PanelConfig,
  xFor: (timestamp: number) => number,
  t0: number,
  timeSpan: number,
  series: { x: number; y: number }[],
) {
  const plotW = WIDTH - MARGIN.left - MARGIN.right;
  const plotH = PANEL_HEIGHT - MARGIN.top - MARGIN.bottom;
  const { yMin, yMax } = cfg;
  const yRange = yMax - yMin || 1;

  const yFor = (v: number) => panelY + MARGIN.top + plotH - ((v - yMin) / yRange) * plotH;

  // grid lines
  ctx.strokeStyle = COLORS.grid;
  ctx.lineWidth = 0.5;
  ctx.fillStyle = COLORS.axisText;
  ctx.font = '9px monospace';
  ctx.textAlign = 'right';
  ctx.textBaseline = 'middle';

  const yTicks = 3;
  for (let i = 0; i <= yTicks; i++) {
    const val = yMin + (yRange * i) / yTicks;
    const y = yFor(val);
    ctx.fillText(val.toFixed(cfg.unit === '°' ? 0 : 1), MARGIN.left - 5, y);
    ctx.beginPath();
    ctx.moveTo(MARGIN.left, y);
    ctx.lineTo(WIDTH - MARGIN.right, y);
    ctx.stroke();
  }

  // baseline (zero reference)
  if (cfg.baseline !== undefined) {
    const y = yFor(cfg.baseline);
    ctx.strokeStyle = COLORS.zeroLine;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(MARGIN.left, y);
    ctx.lineTo(WIDTH - MARGIN.right, y);
    ctx.stroke();
  }

  // border
  ctx.strokeStyle = COLORS.border;
  ctx.lineWidth = 0.5;
  ctx.strokeRect(MARGIN.left, panelY + MARGIN.top, plotW, plotH);

  // panel title + unit
  ctx.fillStyle = cfg.color;
  ctx.font = '10px monospace';
  ctx.textAlign = 'left';
  ctx.textBaseline = 'bottom';
  ctx.fillText(cfg.title, MARGIN.left + 4, panelY + 4);
  ctx.fillStyle = COLORS.axisText;
  ctx.font = '9px monospace';
  ctx.fillText(`(${cfg.unit})`, MARGIN.left + 4 + ctx.measureText(cfg.title).width + 6, panelY + 4);

  // data line
  if (series.length >= 2) {
    ctx.strokeStyle = cfg.color;
    ctx.lineWidth = 1.4;
    ctx.beginPath();
    for (let i = 0; i < series.length; i++) {
      const x = series[i].x;
      const y = yFor(series[i].y);
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();
  }

  // x-axis time labels
  ctx.fillStyle = COLORS.axisText;
  ctx.font = '9px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';
  const xTicks = 4;
  for (let i = 0; i <= xTicks; i++) {
    const t = t0 + (timeSpan * i) / xTicks;
    const x = MARGIN.left + (plotW * i) / xTicks;
    ctx.fillText(`${(t - t0).toFixed(1)}s`, x, panelY + MARGIN.top + plotH + 3);
  }
}

export default function TickStream({ ticks }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, WIDTH, TOTAL_HEIGHT);

    const recent = ticks.slice(-MAX_POINTS);
    if (recent.length < 2) {
      ctx.fillStyle = '#666';
      ctx.font = '12px monospace';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'middle';
      ctx.fillText('Waiting for ticks...', MARGIN.left + 8, TOTAL_HEIGHT / 2);
      return;
    }

    const plotW = WIDTH - MARGIN.left - MARGIN.right;
    const t0 = recent[0].timestamp;
    const tEnd = recent[recent.length - 1].timestamp;
    const timeSpan = tEnd - t0 || 1;
    const xFor = (timestamp: number) => MARGIN.left + ((timestamp - t0) / timeSpan) * plotW;

    // ---- Panel 1: Rate (tic vs tok) ----
    const ticRates: { x: number; y: number }[] = [];
    const tokRates: { x: number; y: number }[] = [];

    for (let i = 0; i < recent.length; i++) {
      const pt = { x: xFor(recent[i].timestamp), y: recent[i].rate_spd };
      if (i % 2 === 0) {
        ticRates.push(pt);
      } else {
        tokRates.push(pt);
      }
    }

    const allRates = recent.map((t) => t.rate_spd);
    const minR = Math.min(0, ...allRates);
    const maxR = Math.max(0, ...allRates);
    const rRange = maxR - minR || 1;
    const pad = rRange * 0.1;
    const rateCfg: PanelConfig = {
      title: 'Rate',
      unit: 's/d',
      color: COLORS.tic,
      yMin: minR - pad,
      yMax: maxR + pad,
      baseline: 0,
    };

    drawPanel(ctx, 0, rateCfg, xFor, t0, timeSpan, []);

    // draw tic and tok lines separately on the rate panel
    if (ticRates.length >= 2) {
      ctx.strokeStyle = COLORS.tic;
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ticRates.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
      ctx.stroke();
    }
    if (tokRates.length >= 2) {
      ctx.strokeStyle = COLORS.tok;
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      tokRates.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
      ctx.stroke();
    }
    // legend
    ctx.font = '9px monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'bottom';
    const plotH = PANEL_HEIGHT - MARGIN.top - MARGIN.bottom;
    const legendY = 0 + MARGIN.top + plotH - 4;
    ctx.fillStyle = COLORS.tic;
    ctx.fillText('— tic', WIDTH - MARGIN.right - 90, legendY);
    ctx.fillStyle = COLORS.tok;
    ctx.fillText('— tok', WIDTH - MARGIN.right - 38, legendY);

    // ---- Panel 2: Amplitude ----
    const ampPts: { x: number; y: number }[] = recent.map((t) => ({
      x: xFor(t.timestamp),
      y: t.amplitude,
    }));
    const maxAmp = Math.max(0.5, ...recent.map((t) => t.amplitude));
    const ampCfg: PanelConfig = {
      title: 'Amplitude',
      unit: '°',
      color: COLORS.amplitude,
      yMin: 0,
      yMax: maxAmp * 1.15,
    };
    drawPanel(ctx, PANEL_HEIGHT + PANEL_GAP, ampCfg, xFor, t0, timeSpan, ampPts);

    // ---- Panel 3: Beat error ----
    const beatPts: { x: number; y: number }[] = [];
    for (let i = 1; i < recent.length; i += 2) {
      if (i + 1 < recent.length) {
        const beMs = Math.abs(recent[i].interval_s - recent[i + 1].interval_s) * 1000;
        beatPts.push({ x: xFor(recent[i].timestamp), y: beMs });
      }
    }
    const maxBe = Math.max(0.05, ...beatPts.map((p) => p.y));
    const beCfg: PanelConfig = {
      title: 'Beat Error',
      unit: 'ms',
      color: COLORS.beatError,
      yMin: 0,
      yMax: maxBe * 1.2,
    };
    drawPanel(ctx, (PANEL_HEIGHT + PANEL_GAP) * 2, beCfg, xFor, t0, timeSpan, beatPts);
  }, [ticks]);

  return (
    <div className="tick-stream">
      <h4>Live Tick Stream</h4>
      <canvas
        ref={canvasRef}
        width={WIDTH}
        height={TOTAL_HEIGHT}
        className="stream-canvas"
      />
    </div>
  );
}