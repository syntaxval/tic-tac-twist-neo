<script lang="ts">
  import type { SearchTelemetry } from '../types';

  interface Props {
    telemetry: SearchTelemetry | null;
    thinking: boolean;
  }

  let { telemetry, thinking }: Props = $props();

  const exactFmt = new Intl.NumberFormat();

  function formatCompact(value: number): string {
    if (value >= 1_000_000) {
      return `${(value / 1_000_000).toFixed(2)}M`;
    }
    if (value >= 1_000) {
      return `${(value / 1_000).toFixed(1)}k`;
    }
    return String(value);
  }
</script>

<section class="telemetry" aria-label="AI search telemetry">
  <dl class="grid">
    <div class="cell">
      <dt>algo</dt>
      <dd class:searching={thinking}>
        {thinking ? 'searching' : (telemetry?.algorithm ?? '-')}
      </dd>
    </div>
    <div class="cell">
      <dt>depth</dt>
      <dd>{telemetry ? telemetry.depthReached : '-'}</dd>
    </div>
    <div
      class="cell"
      title={telemetry ? `${exactFmt.format(telemetry.nodesEvaluated)} nodes` : undefined}
    >
      <dt>nodes</dt>
      <dd>{telemetry ? formatCompact(telemetry.nodesEvaluated) : '-'}</dd>
    </div>
    <div
      class="cell"
      title={telemetry ? `${exactFmt.format(telemetry.ttHits)} transposition table hits` : undefined}
    >
      <dt>tt</dt>
      <dd>{telemetry ? formatCompact(telemetry.ttHits) : '-'}</dd>
    </div>
    <div class="cell">
      <dt>time</dt>
      <dd>{telemetry ? `${telemetry.elapsedMs.toFixed(1)}ms` : '-'}</dd>
    </div>
    <div
      class="cell"
      title={telemetry ? `${exactFmt.format(telemetry.nodesPerSec)} nodes per second` : undefined}
    >
      <dt>rate</dt>
      <dd>{telemetry ? `${formatCompact(telemetry.nodesPerSec)}/s` : '-'}</dd>
    </div>
  </dl>
</section>

<style>
  .telemetry {
    padding: 0.45rem 0.75rem;
    border-radius: 0.6rem;
    border: 1px solid color-mix(in srgb, var(--border) 65%, transparent);
    background: color-mix(in srgb, var(--surface) 55%, transparent);
    color: var(--muted);
    font-size: 0.78rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 0.35rem 0.65rem;
    margin: 0;
  }

  .cell {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.4rem;
    min-width: 0;
    white-space: nowrap;
  }

  dt {
    color: var(--muted);
    opacity: 0.8;
    flex-shrink: 0;
  }

  dd {
    margin: 0;
    color: var(--text);
    opacity: 0.85;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  dd.searching {
    color: var(--accent);
  }

  @media (max-width: 640px) {
    .telemetry {
      padding: 0.55rem 0.8rem;
    }

    .grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 0.4rem 1.25rem;
    }

    .cell {
      padding-block: 0.1rem;
      border-bottom: 1px solid color-mix(in srgb, var(--border) 35%, transparent);
    }

    .cell:nth-last-child(-n + 2) {
      border-bottom: none;
    }
  }
</style>
