<script lang="ts">
  import { untrack } from 'svelte';
  import type { AiAlgorithm, GameConfig, PlayerConfig } from '../types';

  interface Props {
    config: GameConfig;
    onApply: (next: GameConfig) => void;
  }

  interface PlayerDraft {
    name: string;
    symbol: string;
    color: string;
    role: 'human' | 'ai';
    algorithm: AiAlgorithm;
    timeBudgetMs: number;
  }

  function toDrafts(players: PlayerConfig[]): PlayerDraft[] {
    return players.map((p) =>
      p.kind.type === 'ai'
        ? {
            name: p.name,
            symbol: p.symbol,
            color: p.color,
            role: 'ai',
            algorithm: p.kind.algorithm,
            timeBudgetMs: p.kind.timeBudgetMs,
          }
        : {
            name: p.name,
            symbol: p.symbol,
            color: p.color,
            role: 'human',
            algorithm: 'paranoid',
            timeBudgetMs: 120,
          }
    );
  }

  let { config, onApply }: Props = $props();

  let boardSize = $state(untrack(() => config.boardSize));
  let winLength = $state(untrack(() => config.winLength));
  let fourCorners = $state(untrack(() => config.fourCorners));
  let drafts = $state<PlayerDraft[]>(untrack(() => toDrafts(config.players)));
  let activePlayerTab = $state(0);

  $effect(() => {
    if (winLength > boardSize) {
      winLength = boardSize;
    }
  });

  const activeDraft = $derived(drafts[activePlayerTab]);
  const isHuman = $derived(activeDraft.role === 'human');
  const isGreedy = $derived(activeDraft.role === 'ai' && activeDraft.algorithm === 'greedy');

  function formatBudget(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(2).replace(/\.?0+$/, '')} s` : `${ms} ms`;
  }

  function shortRole(d: PlayerDraft): string {
    if (d.role === 'human') return 'Human';
    if (d.algorithm === 'paranoid') return 'Paranoid';
    if (d.algorithm === 'maxN') return 'MaxN';
    return 'Greedy';
  }

  const isDirty = $derived(
    boardSize !== config.boardSize ||
      winLength !== config.winLength ||
      fourCorners !== config.fourCorners ||
      drafts.some((d, idx) => {
        const cur = config.players[idx];
        if (!cur) return true;
        if (d.role !== cur.kind.type) return true;
        if (d.role === 'ai' && cur.kind.type === 'ai') {
          if (d.algorithm !== cur.kind.algorithm) return true;
          if (d.algorithm !== 'greedy' && d.timeBudgetMs !== cur.kind.timeBudgetMs) {
            return true;
          }
        }
        return false;
      })
  );

  function apply() {
    const players: PlayerConfig[] = drafts.map((d) => ({
      name: d.name,
      symbol: d.symbol,
      color: d.color,
      kind:
        d.role === 'ai'
          ? {
              type: 'ai',
              algorithm: d.algorithm,
              timeBudgetMs: d.timeBudgetMs,
              maxDepth: 16,
            }
          : { type: 'human' },
    }));

    onApply({
      boardSize,
      winLength,
      fourCorners,
      players,
    });
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

  <div class="player-section">
    <span class="section-label">Player Settings</span>

    <div class="player-tabs" role="tablist" aria-label="Configure player">
      {#each drafts as d, idx}
        <button
          type="button"
          role="tab"
          aria-selected={activePlayerTab === idx}
          class="player-tab"
          class:selected={activePlayerTab === idx}
          onclick={() => (activePlayerTab = idx)}
        >
          <span class="tab-sym" style:color={d.color}>{d.symbol}</span>
          <span class="tab-name">{d.name}</span>
          <span class="tab-role">{shortRole(d)}</span>
        </button>
      {/each}
    </div>

    <div class="player-pane" role="tabpanel" aria-label={`Settings for ${activeDraft.name}`}>
      <div class="field">
        <label for="player-role">Control</label>
        <select
          id="player-role"
          name="playerRole"
          bind:value={drafts[activePlayerTab].role}
        >
          <option value="ai">AI</option>
          <option value="human">Human</option>
        </select>
      </div>

      <div class="field" class:disabled-field={isHuman}>
        <label for="ai-algo">AI Strategy</label>
        <select
          id="ai-algo"
          name="aiAlgorithm"
          disabled={isHuman}
          bind:value={drafts[activePlayerTab].algorithm}
        >
          <option value="paranoid">Paranoid Alpha-Beta + TT</option>
          <option value="maxN">MaxN (3-Player Vector)</option>
          <option value="greedy">Greedy Heuristic (1-ply)</option>
        </select>
      </div>

      <div class="field" class:disabled-field={isHuman || isGreedy}>
        <label for="ai-budget">
          AI time budget:
          {#if isHuman}
            <strong>Human</strong>
          {:else if isGreedy}
            <strong>N/A (1-ply)</strong>
          {:else}
            <strong>{formatBudget(activeDraft.timeBudgetMs)}</strong>
          {/if}
        </label>
        <input
          id="ai-budget"
          type="range"
          min="20"
          max="2000"
          step="20"
          disabled={isHuman || isGreedy}
          bind:value={drafts[activePlayerTab].timeBudgetMs}
        />
      </div>
    </div>
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
    opacity: 0.45;
  }

  .field.disabled-field input,
  .field.disabled-field select {
    cursor: not-allowed;
  }

  .checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .player-section {
    display: grid;
    gap: 0.6rem;
    padding-top: 0.65rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 65%, transparent);
  }

  .section-label {
    font-size: 0.78rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .player-tabs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.35rem;
  }

  .player-tab {
    display: grid;
    justify-items: center;
    gap: 0.1rem;
    padding: 0.4rem 0.25rem;
    border-radius: 0.55rem;
    border: 1px solid var(--border);
    background: var(--surface-alt);
    font-size: 0.75rem;
    line-height: 1.2;
    transition: border-color 140ms ease, background-color 140ms ease;
  }

  .player-tab:hover {
    border-color: var(--accent);
  }

  .player-tab.selected {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 14%, var(--surface-alt));
  }

  .tab-sym {
    font-size: 0.9rem;
    font-weight: 700;
  }

  .tab-name {
    font-weight: 600;
    color: var(--text);
  }

  .tab-role {
    font-size: 0.66rem;
    color: var(--muted);
  }

  .player-pane {
    display: grid;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 0.7rem;
    border: 1px solid color-mix(in srgb, var(--border) 65%, transparent);
    background: color-mix(in srgb, var(--surface-alt) 55%, transparent);
  }

  select {
    appearance: none;
    width: 100%;
    padding: 0.45rem 2.1rem 0.45rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background-color: var(--surface);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12' fill='none'%3E%3Cpath d='M2.5 4.5L6 8L9.5 4.5' stroke='%2364748b' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.7rem center;
    background-size: 0.85rem;
    cursor: pointer;
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
