<script lang="ts">
  import { untrack } from 'svelte';
  import type { AiAlgorithm, GameConfig } from '../types';

  interface Props {
    config: GameConfig;
    aiAlgorithm: AiAlgorithm;
    timeBudgetMs: number;
    onApply: (next: GameConfig, algo: AiAlgorithm, budget: number) => void;
  }

  let { config, aiAlgorithm, timeBudgetMs, onApply }: Props = $props();

  let boardSize = $state(untrack(() => config.boardSize));
  let winLength = $state(untrack(() => config.winLength));
  let fourCorners = $state(untrack(() => config.fourCorners));
  let algorithm = $state<AiAlgorithm>(untrack(() => aiAlgorithm));
  let budget = $state(untrack(() => timeBudgetMs));
  let playerTypes = $state<('human' | 'ai')[]>(
    untrack(() => config.players.map((p) => p.kind.type))
  );

  $effect(() => {
    if (winLength > boardSize) {
      winLength = boardSize;
    }
  });

  const isGreedy = $derived(algorithm === 'greedy');

  function formatBudget(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(2).replace(/\.?0+$/, '')} s` : `${ms} ms`;
  }

  const isDirty = $derived(
    boardSize !== config.boardSize ||
      winLength !== config.winLength ||
      fourCorners !== config.fourCorners ||
      algorithm !== aiAlgorithm ||
      (!isGreedy && budget !== timeBudgetMs) ||
      playerTypes.some((type, idx) => type !== config.players[idx]?.kind.type)
  );

  function apply() {
    const players = config.players.map((p, idx) => ({
      ...p,
      kind:
        playerTypes[idx] === 'ai'
          ? {
              type: 'ai' as const,
              algorithm,
              timeBudgetMs: budget,
              maxDepth: 16,
            }
          : { type: 'human' as const },
    }));

    onApply(
      {
        boardSize,
        winLength,
        fourCorners,
        players,
      },
      algorithm,
      budget
    );
  }
</script>

<section class="panel" aria-label="Game configuration">
  <h2>Match Setup</h2>

  <div class="field">
    <label for="board-size">Board size: <strong>{boardSize}×{boardSize}</strong></label>
    <input id="board-size" type="range" min="3" max="10" bind:value={boardSize} />
  </div>

  <div class="field">
    <label for="win-length">Win length: <strong>{winLength} in a row</strong></label>
    <input id="win-length" type="range" min="3" max={boardSize} bind:value={winLength} />
  </div>

  <label class="checkbox">
    <input type="checkbox" bind:checked={fourCorners} />
    <span>Enable Four Corners win rule</span>
  </label>

  <div class="field">
    <label for="ai-algo">AI Strategy</label>
    <select id="ai-algo" name="aiAlgorithm" bind:value={algorithm}>
      <option value="paranoid">Paranoid Alpha-Beta + TT</option>
      <option value="maxN">MaxN (3-Player Vector)</option>
      <option value="greedy">Greedy Heuristic (1-ply)</option>
    </select>
  </div>

  <div class="field" class:disabled-field={isGreedy}>
    <label for="ai-budget">
      AI time budget:
      {#if isGreedy}
        <strong>N/A (1-ply)</strong>
      {:else}
        <strong>{formatBudget(budget)}</strong>
      {/if}
    </label>
    <input
      id="ai-budget"
      type="range"
      min="20"
      max="2000"
      step="20"
      disabled={isGreedy}
      bind:value={budget}
    />
  </div>

  <div class="roster">
    <span class="label">Players</span>
    {#each config.players as p, idx}
      <div class="roster-row">
        <span class="badge" style:--c={p.color}>{p.symbol}</span>
        <span class="name">{p.name}</span>
        <select
          name={`player-${idx}-kind`}
          aria-label={`Control mode for ${p.name}`}
          bind:value={playerTypes[idx]}
        >
          <option value="ai">AI</option>
          <option value="human">Human</option>
        </select>
      </div>
    {/each}
  </div>

  <button type="button" class="apply-btn" onclick={apply} disabled={!isDirty}>
    Apply
  </button>
</section>

<style>
  .panel {
    display: grid;
    gap: 0.9rem;
    padding: 1.1rem;
    border-radius: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  h2 {
    margin: 0;
    font-size: 1rem;
    letter-spacing: 0.02em;
  }

  .field {
    display: grid;
    gap: 0.35rem;
    font-size: 0.875rem;
  }

  .field.disabled-field {
    opacity: 0.5;
  }

  .field.disabled-field input {
    cursor: not-allowed;
  }

  .checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  select {
    appearance: none;
    width: 100%;
    padding: 0.45rem 2.1rem 0.45rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background-color: var(--surface-alt);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12' fill='none'%3E%3Cpath d='M2.5 4.5L6 8L9.5 4.5' stroke='%2364748b' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.7rem center;
    background-size: 0.85rem;
    cursor: pointer;
  }

  .roster-row select {
    min-width: 6.75rem;
  }

  @supports (appearance: base-select) {
    select,
    select::picker(select) {
      appearance: base-select;
    }

    select {
      display: inline-flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
      padding: 0.45rem 0.75rem;
      background-image: none;
    }

    select::picker-icon {
      color: var(--muted);
      margin-inline-start: auto;
      transition: rotate 150ms ease;
    }

    select:open::picker-icon {
      rotate: 180deg;
    }

    select::picker(select) {
      margin-top: 0.25rem;
      padding: 0.25rem;
      border-radius: 0.6rem;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--text);
      box-shadow: 0 10px 24px rgb(0 0 0 / 0.16);
    }

    select option {
      padding: 0.4rem 0.6rem;
      border-radius: 0.4rem;
      cursor: pointer;
    }

    select option:hover {
      background: var(--surface-alt);
    }
  }

  .roster {
    display: grid;
    gap: 0.45rem;
  }

  .roster .label {
    font-size: 0.8rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .roster-row {
    display: grid;
    grid-template-columns: 1.75rem 1fr auto;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .badge {
    display: grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 0.45rem;
    background: var(--surface-alt);
    color: var(--c);
    font-weight: 700;
  }

  .apply-btn {
    padding: 0.65rem 0.9rem;
    border: none;
    border-radius: 0.65rem;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }

  .apply-btn:not(:disabled):hover {
    filter: brightness(1.08);
  }

  .apply-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
