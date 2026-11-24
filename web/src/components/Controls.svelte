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

  function apply() {
    const players = config.players.map((p, idx) => ({
      ...p,
      kind:
        playerTypes[idx] === 'ai'
          ? {
              type: 'ai' as const,
              algorithm,
              timeBudgetMs: budget,
              maxDepth: 12,
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
    <select id="ai-algo" bind:value={algorithm}>
      <option value="paranoid">Paranoid Alpha-Beta + TT</option>
      <option value="maxN">MaxN (3-Player Vector)</option>
      <option value="greedy">Greedy Heuristic (1-ply)</option>
    </select>
  </div>

  <div class="field">
    <label for="ai-budget">AI time budget: <strong>{budget} ms</strong></label>
    <input id="ai-budget" type="range" min="20" max="500" step="20" bind:value={budget} />
  </div>

  <div class="roster">
    <span class="label">Players</span>
    {#each config.players as p, idx}
      <div class="roster-row">
        <span class="badge" style:--c={p.color}>{p.symbol}</span>
        <span class="name">{p.name}</span>
        <select
          aria-label={`Control mode for ${p.name}`}
          bind:value={playerTypes[idx]}
        >
          <option value="ai">AI</option>
          <option value="human">Human</option>
        </select>
      </div>
    {/each}
  </div>

  <button type="button" class="apply-btn" onclick={apply}>
    Apply &amp; Start New Game
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

  .checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  select {
    padding: 0.45rem 0.6rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background: var(--surface-alt);
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

  .apply-btn:hover {
    filter: brightness(1.08);
  }
</style>
