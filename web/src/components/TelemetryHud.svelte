<script lang="ts">
  import type { SearchTelemetry } from '../types';

  interface Props {
    telemetry: SearchTelemetry | null;
    thinking: boolean;
  }

  let { telemetry, thinking }: Props = $props();

  const fmt = new Intl.NumberFormat();
</script>

<section class="hud" aria-label="AI search telemetry">
  <div class="hud-header">
    <h2>AI Telemetry (Rust → WASM)</h2>
    {#if thinking}
      <span class="pill active">Searching…</span>
    {:else if telemetry}
      <span class="pill">{telemetry.algorithm}</span>
    {/if}
  </div>

  {#if telemetry}
    <dl class="metrics">
      <div>
        <dt>Depth</dt>
        <dd>{telemetry.depthReached}</dd>
      </div>
      <div>
        <dt>Nodes</dt>
        <dd>{fmt.format(telemetry.nodesEvaluated)}</dd>
      </div>
      <div>
        <dt>TT Hits</dt>
        <dd>{fmt.format(telemetry.ttHits)}</dd>
      </div>
      <div>
        <dt>Time</dt>
        <dd>{telemetry.elapsedMs.toFixed(1)} ms</dd>
      </div>
      <div>
        <dt>Throughput</dt>
        <dd>{fmt.format(telemetry.nodesPerSec)} n/s</dd>
      </div>
    </dl>
  {:else}
    <p class="empty">Waiting for the first AI move…</p>
  {/if}
</section>

<style>
  .hud {
    padding: 1.1rem;
    border-radius: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .hud-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  .pill {
    padding: 0.15rem 0.55rem;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--surface-alt);
    border: 1px solid var(--border);
  }

  .pill.active {
    border-color: var(--accent);
    color: var(--accent);
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(92px, 1fr));
    gap: 0.6rem;
    margin: 0;
  }

  .metrics div {
    padding: 0.55rem 0.65rem;
    border-radius: 0.6rem;
    background: var(--surface-alt);
  }

  dt {
    font-size: 0.72rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  dd {
    margin: 0.15rem 0 0;
    font-size: 0.95rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .empty {
    margin: 0;
    font-size: 0.875rem;
    color: var(--muted);
  }
</style>
