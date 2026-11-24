<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Board from './components/Board.svelte';
  import Controls from './components/Controls.svelte';
  import TelemetryHud from './components/TelemetryHud.svelte';
  import { EngineClient } from './lib/engineClient';
  import type {
    AiAlgorithm,
    GameConfig,
    GameSnapshot,
    SearchTelemetry,
  } from './types';

  const initialConfig: GameConfig = {
    boardSize: 5,
    winLength: 4,
    fourCorners: false,
    players: [
      {
        name: 'Zalgo',
        symbol: '❤',
        color: '#e879f9',
        kind: { type: 'ai', algorithm: 'paranoid', timeBudgetMs: 120, maxDepth: 12 },
      },
      {
        name: 'Brent',
        symbol: '✹',
        color: '#38bdf8',
        kind: { type: 'human' },
      },
      {
        name: 'Josh',
        symbol: '✤',
        color: '#4ade80',
        kind: { type: 'human' },
      },
    ],
  };

  let client: EngineClient | null = null;
  let config = $state<GameConfig>(initialConfig);
  let aiAlgorithm = $state<AiAlgorithm>('paranoid');
  let timeBudgetMs = $state(120);
  let snapshot = $state<GameSnapshot | null>(null);
  let telemetry = $state<SearchTelemetry | null>(null);
  let thinking = $state(false);
  let errorMsg = $state<string | null>(null);
  let runToken = 0;

  async function driveAiTurns(token: number) {
    while (
      client &&
      token === runToken &&
      snapshot &&
      snapshot.outcome.status === 'ongoing' &&
      snapshot.players[snapshot.currentTurn].kind.type === 'ai'
    ) {
      thinking = true;
      try {
        // Brief yield so the UI paints the previous move cleanly.
        await new Promise((r) => setTimeout(r, 80));
        if (token !== runToken) return;
        const resp = await client.playAiTurn();
        if (token !== runToken) return;
        snapshot = resp.state;
        telemetry = resp.result.telemetry;
      } catch (err) {
        errorMsg = err instanceof Error ? err.message : String(err);
        break;
      } finally {
        if (token === runToken) {
          thinking = false;
        }
      }
    }
  }

  async function startNewGame(nextConfig: GameConfig) {
    if (!client) return;
    const token = ++runToken;
    errorMsg = null;
    telemetry = null;
    thinking = false;
    try {
      snapshot = await client.init(nextConfig);
      await driveAiTurns(token);
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handlePlay(cell: number) {
    if (!client || thinking || !snapshot) return;
    const token = ++runToken;
    errorMsg = null;
    try {
      snapshot = await client.playMove(cell);
      await driveAiTurns(token);
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleUndo() {
    if (!client || thinking) return;
    ++runToken;
    errorMsg = null;
    try {
      snapshot = await client.undo();
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleReset() {
    if (!client) return;
    const token = ++runToken;
    errorMsg = null;
    telemetry = null;
    thinking = false;
    snapshot = await client.reset();
    await driveAiTurns(token);
  }

  function handleApply(next: GameConfig, algo: AiAlgorithm, budget: number) {
    config = next;
    aiAlgorithm = algo;
    timeBudgetMs = budget;
    startNewGame(next);
  }

  onMount(() => {
    client = new EngineClient();
    startNewGame(config);
  });

  onDestroy(() => {
    ++runToken;
    client?.destroy();
  });
</script>

<main class="layout">
  <header class="topbar">
    <div>
      <h1>Tic-Tac-Twist Neo</h1>
      <p class="subtitle">u128 Bitboard Engine · Rust → WASM Worker · Svelte 5</p>
    </div>
    <div class="actions">
      <button type="button" onclick={handleUndo} disabled={thinking || !snapshot?.history.length}>
        Undo
      </button>
      <button type="button" onclick={handleReset} disabled={thinking}>
        Reset
      </button>
    </div>
  </header>

  {#if errorMsg}
    <p class="error" role="alert">{errorMsg}</p>
  {/if}

  <div class="content">
    <div class="stage">
      {#if snapshot}
        {@const active = snapshot.players[snapshot.currentTurn]}
        <div class="status-bar" aria-live="polite">
          {#if snapshot.outcome.status === 'win'}
            {@const winner = snapshot.players[snapshot.outcome.player]}
            <span class="dot" style:background={winner.color}></span>
            <strong>Winner: {winner.name} ({winner.symbol})</strong>
            <span class="tag">{snapshot.outcome.kind}</span>
          {:else if snapshot.outcome.status === 'draw'}
            <strong>It's a draw!</strong>
          {:else}
            <span class="dot" style:background={active.color}></span>
            <span>
              Turn: <strong>{active.name} ({active.symbol})</strong>
              {#if active.kind.type === 'ai'}
                <span class="tag">AI</span>
              {/if}
            </span>
          {/if}
        </div>

        <Board {snapshot} disabled={thinking} onPlay={handlePlay} />
        <TelemetryHud {telemetry} {thinking} />
      {:else}
        <p class="loading">Loading Rust WASM engine…</p>
      {/if}
    </div>

    <aside class="sidebar">
      <Controls {config} {aiAlgorithm} {timeBudgetMs} onApply={handleApply} />
    </aside>
  </div>
</main>

<style>
  .layout {
    max-width: 1020px;
    margin: 0 auto;
    padding: 1.25rem;
    display: grid;
    gap: 1.25rem;
  }

  .topbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  h1 {
    margin: 0;
    font-size: clamp(1.35rem, 2.5vw, 1.85rem);
  }

  .subtitle {
    margin: 0.15rem 0 0;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .actions button {
    padding: 0.45rem 0.9rem;
    border-radius: 0.55rem;
    border: 1px solid var(--border);
    background: var(--surface);
    font-weight: 600;
  }

  .actions button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .content {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 1.25rem;
    align-items: start;
  }

  @media (max-width: 820px) {
    .content {
      grid-template-columns: 1fr;
    }
  }

  .stage {
    display: grid;
    gap: 1rem;
  }

  .status-bar {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.7rem 1rem;
    border-radius: 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .dot {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 999px;
  }

  .tag {
    margin-left: 0.35rem;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
    font-size: 0.75rem;
    background: var(--surface-alt);
    border: 1px solid var(--border);
  }

  .error {
    margin: 0;
    padding: 0.75rem 1rem;
    border-radius: 0.65rem;
    background: #7f1d1d;
    color: #fee2e2;
  }

  .loading {
    color: var(--muted);
  }
</style>
