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
  {#if telemetry}
    <span class="algo" class:searching={thinking}>
      {thinking ? 'searching' : telemetry.algorithm}
    </span>
    <dl class="items">
      <div class="item">
        <dt>depth</dt>
        <dd>{telemetry.depthReached}</dd>
      </div>
      <div class="item" title={`${exactFmt.format(telemetry.nodesEvaluated)} nodes`}>
        <dt>nodes</dt>
        <dd>{formatCompact(telemetry.nodesEvaluated)}</dd>
      </div>
      <div class="item" title={`${exactFmt.format(telemetry.ttHits)} transposition table hits`}>
        <dt>tt</dt>
        <dd>{formatCompact(telemetry.ttHits)}</dd>
      </div>
      <div class="item">
        <dt>time</dt>
        <dd>{telemetry.elapsedMs.toFixed(1)}ms</dd>
      </div>
      <div class="item" title={`${exactFmt.format(telemetry.nodesPerSec)} nodes per second`}>
        <dt>rate</dt>
        <dd>{formatCompact(telemetry.nodesPerSec)}/s</dd>
      </div>
    </dl>
  {:else}
    <span class="placeholder">
      {thinking ? 'AI searching...' : 'AI telemetry appears after the first AI move.'}
    </span>
  {/if}
</section>

<style>
  .telemetry {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem 1rem;
    min-height: 2.25rem;
    padding: 0.45rem 0.75rem;
    border-radius: 0.6rem;
    border: 1px solid color-mix(in srgb, var(--border) 65%, transparent);
    background: color-mix(in srgb, var(--surface) 55%, transparent);
    color: var(--muted);
    font-size: 0.78rem;
  }

  .algo {
    font-weight: 600;
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .algo.searching {
    color: var(--accent);
  }

  .items {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem 0.9rem;
    margin: 0;
  }

  .item {
    display: inline-flex;
    align-items: baseline;
    gap: 0.3rem;
    white-space: nowrap;
  }

  dt {
    color: var(--muted);
    opacity: 0.8;
  }

  dd {
    margin: 0;
    color: var(--text);
    opacity: 0.85;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .placeholder {
    opacity: 0.8;
    white-space: nowrap;
  }
</style>
