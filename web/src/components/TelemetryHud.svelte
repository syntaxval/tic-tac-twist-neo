<script lang="ts">
  import type { AiAlgorithm, SearchTelemetry } from '../types';

  interface Props {
    telemetry: SearchTelemetry | null;
    history: SearchTelemetry[];
    turnCount: number;
    thinking: boolean;
    aiAlgorithm: AiAlgorithm;
    timeBudgetMs: number;
  }

  let { telemetry, history, turnCount, thinking, aiAlgorithm, timeBudgetMs }: Props = $props();

  let showGuide = $state(false);
  const exactFmt = new Intl.NumberFormat();

  function formatCompact(value: number): string {
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
    if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
    return String(value);
  }

  function algoLabel(algo: AiAlgorithm): string {
    if (algo === 'paranoid') return 'Paranoid';
    if (algo === 'maxN') return 'MaxN';
    return 'Greedy';
  }

  // Semicircle arc geometry for speedometer gauge (radius 34, length PI * 34 ~= 106.81)
  const ARC_RADIUS = 34;
  const ARC_LENGTH = Math.PI * ARC_RADIUS;

  // Map throughput (nodes/sec) logarithmically from 1k/s..2M/s into 0..1
  const speedRatio = $derived.by(() => {
    if (!telemetry || telemetry.nodesPerSec <= 0) return 0;
    const logMin = Math.log10(1_000);
    const logMax = Math.log10(2_000_000);
    const ratio = (Math.log10(Math.max(1_000, telemetry.nodesPerSec)) - logMin) / (logMax - logMin);
    return Math.min(1, Math.max(0.06, ratio));
  });

  // Shift color smoothly from slate-blue (215deg) toward bright emerald (148deg) as ratio climbs
  const speedColor = $derived.by(() => {
    if (!telemetry) return 'var(--border)';
    const hue = Math.round(215 - speedRatio * 67);
    const sat = Math.round(45 + speedRatio * 35);
    return `hsl(${hue} ${sat}% 48%)`;
  });

  // Lookahead depth ratio (0..1 across 1..10 plies) with greener hue as depth grows
  const depthRatio = $derived.by(() => {
    if (!telemetry) return 0;
    return Math.min(1, Math.max(0.1, telemetry.depthReached / 10));
  });

  const depthColor = $derived.by(() => {
    if (!telemetry) return 'var(--border)';
    const hue = Math.round(210 - depthRatio * 65);
    const sat = Math.round(50 + depthRatio * 30);
    return `hsl(${hue} ${sat}% 48%)`;
  });

  const activeAlgo = $derived(telemetry?.algorithm ?? aiAlgorithm);
  const isGreedy = $derived(activeAlgo === 'greedy');

  function formatMs(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(2)}s` : `${ms.toFixed(1)}ms`;
  }

  function formatBudgetShort(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(2).replace(/\.?0+$/, '')}s` : `${ms}ms`;
  }

  // Think-time budget usage ratio (0..1): greener when fast, shifts toward amber near budget
  const budgetRatio = $derived.by(() => {
    if (!telemetry) return 0;
    if (isGreedy) return 0.04;
    if (timeBudgetMs <= 0) return 0;
    return Math.min(1, Math.max(0.03, telemetry.elapsedMs / timeBudgetMs));
  });

  const budgetColor = $derived.by(() => {
    if (!telemetry) return 'var(--border)';
    const hue = Math.round(148 - budgetRatio * 108); // emerald 148 -> amber 40
    return `hsl(${hue} 76% 46%)`;
  });

  // Cache hit ratio (transposition table reuse)
  const cacheRate = $derived.by(() => {
    if (!telemetry || telemetry.nodesEvaluated === 0) return 0;
    return Math.min(100, Math.round((telemetry.ttHits / telemetry.nodesEvaluated) * 100));
  });

  // Sparkline points (log10 of nodes evaluated across recent AI moves)
  const sparkData = $derived.by(() => {
    const W = 132;
    const H = 36;
    const padX = 4;
    const padY = 5;

    if (history.length === 0) {
      return { line: '', area: '', lastX: 0, lastY: 0, count: 0 };
    }

    const vals = history.map((h) => Math.log10(Math.max(1, h.nodesEvaluated)));
    const maxVal = Math.max(2, ...vals);
    const stepX = history.length > 1 ? (W - padX * 2) / (history.length - 1) : 0;

    const pts = vals.map((v, idx) => {
      const x = history.length === 1 ? W - padX : padX + idx * stepX;
      const y = H - padY - (v / maxVal) * (H - padY * 2);
      return { x, y };
    });

    const line = pts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
    const first = pts[0];
    const last = pts[pts.length - 1];
    const area =
      pts.length > 1
        ? `${line} L${last.x.toFixed(1)},${H} L${first.x.toFixed(1)},${H} Z`
        : '';

    return { line, area, lastX: last.x, lastY: last.y, count: pts.length };
  });
</script>

<section class="hud" aria-label="AI search telemetry">
  <header class="hud-head">
    <div class="title-group">
      <span class="status-dot" class:pulsing={thinking} class:active={!!telemetry}></span>
      <span class="title">AI Engine</span>
      <span class="badge">{algoLabel(telemetry?.algorithm ?? aiAlgorithm)}</span>
    </div>
    <button
      type="button"
      class="guide-toggle"
      aria-expanded={showGuide}
      aria-controls="telemetry-guide"
      onclick={() => (showGuide = !showGuide)}
    >
      {showGuide ? 'Hide guide' : 'What is this?'}
    </button>
  </header>

  <div class="instruments">
    <!-- 1. Speedometer Gauge -->
    <div
      class="gauge-card"
      title={telemetry
        ? `${exactFmt.format(telemetry.nodesPerSec)} board positions checked per second`
        : 'AI throughput gauge'}
    >
      <svg class="gauge-svg" viewBox="0 0 88 50" aria-hidden="true">
        <path
          class="gauge-track"
          d="M 10 44 A 34 34 0 0 1 78 44"
          fill="none"
          stroke-width="6.5"
          stroke-linecap="round"
        />
        <path
          class="gauge-arc"
          d="M 10 44 A 34 34 0 0 1 78 44"
          fill="none"
          stroke={speedColor}
          stroke-width="6.5"
          stroke-linecap="round"
          stroke-dasharray={ARC_LENGTH}
          stroke-dashoffset={ARC_LENGTH * (1 - speedRatio)}
        />
      </svg>
      <div class="gauge-readout">
        <span class="gauge-val">
          {telemetry ? `${formatCompact(telemetry.nodesPerSec)}/s` : '-'}
        </span>
        <span class="gauge-label">Speed</span>
      </div>
    </div>

    <!-- 2. Depth + Think-Time Progress Bars -->
    <div class="bars-card">
      <div class="bar-row">
        <div class="bar-meta">
          <span class="bar-label">Lookahead</span>
          <span class="bar-val">
            {telemetry ? `${telemetry.depthReached} ${telemetry.depthReached === 1 ? 'ply' : 'plies'}` : '-'}
          </span>
        </div>
        <div class="track">
          <div
            class="fill"
            style:width={`${depthRatio * 100}%`}
            style:background={depthColor}
          ></div>
        </div>
      </div>

      <div class="bar-row">
        <div class="bar-meta">
          <span class="bar-label">Think time</span>
          <span class="bar-val">
            {#if isGreedy}
              {telemetry ? `${formatMs(telemetry.elapsedMs)} (1-ply)` : '1-ply'}
            {:else}
              {telemetry
                ? `${formatMs(telemetry.elapsedMs)} / ${formatBudgetShort(timeBudgetMs)}`
                : `- / ${formatBudgetShort(timeBudgetMs)}`}
            {/if}
          </span>
        </div>
        <div class="track">
          <div
            class="fill"
            style:width={`${budgetRatio * 100}%`}
            style:background={budgetColor}
          ></div>
        </div>
      </div>

      <div class="sub-stats">
        <span
          title={telemetry
            ? `${exactFmt.format(telemetry.ttHits)} repeated positions served from cache (${cacheRate}%)`
            : 'Cached positions reused during search'}
        >
          Cache hits <strong>{telemetry ? formatCompact(telemetry.ttHits) : '-'}</strong>
        </span>
        <span>
          Turns <strong>{turnCount}</strong>
        </span>
      </div>
    </div>

    <!-- 3. Positions Evaluated Sparkline -->
    <div
      class="spark-card"
      title={telemetry
        ? `${exactFmt.format(telemetry.nodesEvaluated)} positions evaluated on last AI turn`
        : 'Positions evaluated per AI turn'}
    >
      <div class="spark-meta">
        <span class="bar-label">Positions</span>
        <span class="bar-val">
          {telemetry ? formatCompact(telemetry.nodesEvaluated) : '-'}
        </span>
      </div>
      <svg class="spark-svg" viewBox="0 0 132 36" preserveAspectRatio="none" aria-hidden="true">
        <defs>
          <linearGradient id="sparkFill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#22c55e" stop-opacity="0.32" />
            <stop offset="100%" stop-color="#22c55e" stop-opacity="0.02" />
          </linearGradient>
        </defs>
        <line x1="0" y1="35" x2="132" y2="35" class="spark-baseline" />
        {#if sparkData.area}
          <path d={sparkData.area} fill="url(#sparkFill)" />
        {/if}
        {#if sparkData.line}
          <path
            d={sparkData.line}
            fill="none"
            stroke="#22c55e"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <circle cx={sparkData.lastX} cy={sparkData.lastY} r="2.6" fill="#22c55e" />
        {/if}
      </svg>
      <span class="spark-caption">
        {sparkData.count > 0 ? `Last ${sparkData.count} AI ${sparkData.count === 1 ? 'move' : 'moves'}` : 'Waiting for AI'}
      </span>
    </div>
  </div>

  {#if showGuide}
    <dl id="telemetry-guide" class="guide">
      <div>
        <dt>Speed</dt>
        <dd>Board positions the Rust WebAssembly engine evaluates per second. Greener means higher throughput.</dd>
      </div>
      <div>
        <dt>Lookahead</dt>
        <dd>How many turns ahead (plies) the AI searched before picking its move.</dd>
      </div>
      <div>
        <dt>Think time</dt>
        <dd>Milliseconds spent out of the configured time budget. Instant tactical wins or blocks finish in under 1ms.</dd>
      </div>
      <div>
        <dt>Positions</dt>
        <dd>Total future board states evaluated on the last move, plotted across recent AI turns.</dd>
      </div>
      <div>
        <dt>Cache hits</dt>
        <dd>Repeated board states reused from the Zobrist transposition table instead of re-searching.</dd>
      </div>
    </dl>
  {/if}
</section>

<style>
  .hud {
    display: grid;
    gap: 0.65rem;
    padding: 0.7rem 0.85rem;
    border-radius: 0.85rem;
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    background: color-mix(in srgb, var(--surface) 70%, transparent);
    font-size: 0.78rem;
    color: var(--muted);
  }

  .hud-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .title-group {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
  }

  .status-dot {
    width: 0.48rem;
    height: 0.48rem;
    border-radius: 999px;
    background: var(--border);
    transition: background-color 200ms ease;
  }

  .status-dot.active {
    background: #22c55e;
  }

  .status-dot.pulsing {
    background: var(--accent);
    animation: pulse 900ms ease-in-out infinite;
  }

  .title {
    font-weight: 600;
    color: var(--text);
    opacity: 0.88;
  }

  .badge {
    padding: 0.08rem 0.45rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-alt);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--muted);
  }

  .guide-toggle {
    padding: 0.12rem 0.45rem;
    border: none;
    border-radius: 0.4rem;
    background: transparent;
    color: var(--muted);
    font-size: 0.72rem;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .guide-toggle:hover {
    color: var(--text);
  }

  .instruments {
    display: grid;
    grid-template-columns: 108px minmax(0, 1fr) 136px;
    gap: 0.8rem;
    align-items: center;
  }

  /* Speedometer gauge */
  .gauge-card {
    position: relative;
    display: grid;
    place-items: center;
    height: 70px;
  }

  .gauge-svg {
    width: 96px;
    height: 56px;
    overflow: visible;
  }

  .gauge-track {
    stroke: color-mix(in srgb, var(--border) 65%, transparent);
  }

  .gauge-arc {
    transition:
      stroke-dashoffset 280ms cubic-bezier(0.22, 1, 0.36, 1),
      stroke 280ms ease;
  }

  .gauge-readout {
    position: absolute;
    bottom: 2px;
    display: grid;
    justify-items: center;
    line-height: 1.15;
  }

  .gauge-val {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .gauge-label {
    font-size: 0.67rem;
    color: var(--muted);
  }

  /* Progress bars */
  .bars-card {
    display: grid;
    gap: 0.42rem;
    min-width: 0;
  }

  .bar-row {
    display: grid;
    gap: 0.2rem;
  }

  .bar-meta,
  .spark-meta,
  .sub-stats {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.4rem;
    white-space: nowrap;
  }

  .bar-label {
    font-size: 0.72rem;
    color: var(--muted);
  }

  .bar-val {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 6px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--border) 55%, transparent);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: 999px;
    transition:
      width 280ms cubic-bezier(0.22, 1, 0.36, 1),
      background-color 280ms ease;
  }

  .sub-stats {
    font-size: 0.7rem;
    color: var(--muted);
    padding-top: 0.05rem;
  }

  .sub-stats strong {
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  /* Sparkline */
  .spark-card {
    display: grid;
    gap: 0.2rem;
    height: 70px;
    align-content: space-between;
  }

  .spark-svg {
    width: 100%;
    height: 36px;
    display: block;
  }

  .spark-baseline {
    stroke: color-mix(in srgb, var(--border) 55%, transparent);
    stroke-width: 1;
  }

  .spark-caption {
    font-size: 0.67rem;
    color: var(--muted);
    text-align: right;
    white-space: nowrap;
  }

  /* Plain-English guide */
  .guide {
    display: grid;
    gap: 0.4rem;
    margin: 0;
    padding-top: 0.6rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 55%, transparent);
    font-size: 0.74rem;
    line-height: 1.4;
  }

  .guide div {
    display: grid;
    grid-template-columns: 112px minmax(0, 1fr);
    gap: 0.5rem;
  }

  .guide dt {
    font-weight: 600;
    color: var(--text);
  }

  .guide dd {
    margin: 0;
    color: var(--muted);
  }

  @media (max-width: 560px) {
    .instruments {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 0.7rem 0.9rem;
    }

    .bars-card {
      grid-column: 1 / -1;
      order: 3;
    }

    .guide div {
      grid-template-columns: 1fr;
      gap: 0.1rem;
    }
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.35;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .gauge-arc,
    .fill,
    .status-dot {
      transition: none;
      animation: none;
    }
  }
</style>
