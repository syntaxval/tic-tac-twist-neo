<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Board from './components/Board.svelte';
  import Controls from './components/Controls.svelte';
  import TelemetryHud from './components/TelemetryHud.svelte';
  import Toast from './components/Toast.svelte';
  import { EngineClient } from './lib/engineClient';
  import type {
    GameConfig,
    GameSnapshot,
    PlayerTelemetryState,
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
        kind: { type: 'ai', algorithm: 'paranoid', timeBudgetMs: 120, maxDepth: 16 },
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

  function emptyStats(count: number): PlayerTelemetryState[] {
    return Array.from({ length: count }, () => ({
      turnCount: 0,
      totalTimeMs: 0,
      aiHistory: [],
      totalNodes: 0,
      totalTtHits: 0,
      peakNodesPerSec: 0,
      depthSum: 0,
      humanMoveTimesMs: [],
      fastestHumanMs: null,
      slowestHumanMs: null,
    }));
  }

  let client: EngineClient | null = null;
  let config = $state<GameConfig>(initialConfig);
  let snapshot = $state<GameSnapshot | null>(null);
  let playerStats = $state<PlayerTelemetryState[]>(emptyStats(initialConfig.players.length));
  let resetKey = $state(0);
  let thinking = $state(false);
  let started = $state(false);
  let errorMsg = $state<string | null>(null);
  let toastMessage = $state<string | null>(null);
  let toastKey = $state(0);
  let runToken = 0;
  let humanTurnStartedAt: number | null = null;

  function recordAiTurn(playerIdx: number, t: SearchTelemetry) {
    const prev = playerStats[playerIdx];
    if (!prev) return;
    const updated: PlayerTelemetryState = {
      ...prev,
      turnCount: prev.turnCount + 1,
      totalTimeMs: prev.totalTimeMs + t.elapsedMs,
      aiHistory: [...prev.aiHistory.slice(-15), t],
      totalNodes: prev.totalNodes + t.nodesEvaluated,
      totalTtHits: prev.totalTtHits + t.ttHits,
      peakNodesPerSec: Math.max(prev.peakNodesPerSec, t.nodesPerSec),
      depthSum: prev.depthSum + t.depthReached,
    };
    playerStats = playerStats.map((s, i) => (i === playerIdx ? updated : s));
  }

  function recordHumanTurn(playerIdx: number, elapsedMs: number) {
    const prev = playerStats[playerIdx];
    if (!prev) return;
    const clamped = Math.max(1, elapsedMs);
    const updated: PlayerTelemetryState = {
      ...prev,
      turnCount: prev.turnCount + 1,
      totalTimeMs: prev.totalTimeMs + clamped,
      humanMoveTimesMs: [...prev.humanMoveTimesMs.slice(-15), clamped],
      fastestHumanMs:
        prev.fastestHumanMs === null ? clamped : Math.min(prev.fastestHumanMs, clamped),
      slowestHumanMs:
        prev.slowestHumanMs === null ? clamped : Math.max(prev.slowestHumanMs, clamped),
    };
    playerStats = playerStats.map((s, i) => (i === playerIdx ? updated : s));
  }

  function markNextHumanClock() {
    if (
      snapshot &&
      snapshot.outcome.status === 'ongoing' &&
      snapshot.players[snapshot.currentTurn].kind.type === 'human'
    ) {
      humanTurnStartedAt = performance.now();
    } else {
      humanTurnStartedAt = null;
    }
  }

  async function driveAiTurns(token: number) {
    while (
      client &&
      token === runToken &&
      started &&
      snapshot &&
      snapshot.outcome.status === 'ongoing' &&
      snapshot.players[snapshot.currentTurn].kind.type === 'ai'
    ) {
      const actingPlayer = snapshot.currentTurn;
      thinking = true;
      try {
        await new Promise((r) => setTimeout(r, 80));
        if (token !== runToken) return;
        const resp = await client.playAiTurn();
        if (token !== runToken) return;
        snapshot = resp.state;
        recordAiTurn(actingPlayer, resp.result.telemetry);
      } catch (err) {
        errorMsg = err instanceof Error ? err.message : String(err);
        break;
      } finally {
        if (token === runToken) {
          thinking = false;
        }
      }
    }
    if (token === runToken) {
      markNextHumanClock();
    }
  }

  async function initBoard(nextConfig: GameConfig, autoStart: boolean): Promise<boolean> {
    if (!client) return false;
    const token = ++runToken;
    errorMsg = null;
    playerStats = emptyStats(nextConfig.players.length);
    resetKey += 1;
    thinking = false;
    started = autoStart;
    humanTurnStartedAt = null;
    try {
      snapshot = await client.init($state.snapshot(nextConfig));
      if (autoStart) {
        await driveAiTurns(token);
      } else {
        markNextHumanClock();
      }
      return true;
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
      return false;
    }
  }

  async function handleStart() {
    if (!client || thinking) return;
    const token = ++runToken;
    errorMsg = null;
    playerStats = emptyStats(config.players.length);
    started = true;
    humanTurnStartedAt = null;
    try {
      if (!snapshot || snapshot.history.length > 0 || snapshot.outcome.status !== 'ongoing') {
        snapshot = await client.reset();
      }
      await driveAiTurns(token);
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handlePlay(cell: number) {
    if (!client || thinking || !snapshot) return;
    const actingPlayer = snapshot.currentTurn;
    if (!started && snapshot.players[actingPlayer].kind.type === 'ai') {
      return;
    }
    const elapsed =
      humanTurnStartedAt !== null ? performance.now() - humanTurnStartedAt : 0;
    const token = ++runToken;
    started = true;
    errorMsg = null;
    try {
      snapshot = await client.playMove(cell);
      if (elapsed > 0) {
        recordHumanTurn(actingPlayer, elapsed);
      }
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
      markNextHumanClock();
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleReset() {
    if (!client) return;
    ++runToken;
    errorMsg = null;
    playerStats = emptyStats(config.players.length);
    thinking = false;
    started = false;
    humanTurnStartedAt = null;
    snapshot = await client.reset();
    markNextHumanClock();
  }

  async function handleApply(next: GameConfig) {
    const ok = await initBoard(next, false);
    if (ok) {
      config = next;
      toastMessage = `Settings applied (${next.boardSize}x${next.boardSize}, ${next.winLength} in a row).`;
      toastKey += 1;
    }
  }

  onMount(() => {
    client = new EngineClient();
    initBoard(config, false);
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
      <p class="subtitle">u128 Bitboard Engine · Rust WASM Worker · Svelte 5</p>
    </div>
    <div class="actions">
      <button type="button" onclick={handleUndo} disabled={thinking || !snapshot?.history.length}>
        Undo
      </button>
      <button type="button" onclick={handleReset} disabled={thinking}>
        Reset
      </button>
      <button type="button" class="primary" onclick={handleStart} disabled={thinking}>
        Start
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
        {@const awaitingStart = !started && active.kind.type === 'ai' && snapshot.outcome.status === 'ongoing'}
        <div class="status-bar" aria-live="polite">
          {#if snapshot.outcome.status === 'win'}
            {@const winner = snapshot.players[snapshot.outcome.player]}
            <span class="dot" style:background={winner.color}></span>
            <strong>Winner: {winner.name} ({winner.symbol})</strong>
            <span class="tag">{snapshot.outcome.kind}</span>
          {:else if snapshot.outcome.status === 'draw'}
            <strong>It's a draw!</strong>
          {:else if awaitingStart}
            <span class="dot" style:background={active.color}></span>
            <span>
              Ready: <strong>{active.name} ({active.symbol})</strong> moves first. Press Start.
            </span>
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

        <Board {snapshot} disabled={thinking || awaitingStart} onPlay={handlePlay} />
        <TelemetryHud
          players={snapshot.players}
          {playerStats}
          currentTurn={snapshot.currentTurn}
          {thinking}
          outcome={snapshot.outcome}
          {resetKey}
        />
      {:else}
        <p class="loading">Loading Rust WASM engine...</p>
      {/if}
    </div>

    <aside class="sidebar">
      <Controls {config} onApply={handleApply} />
    </aside>
  </div>
</main>

<Toast
  message={toastMessage}
  {toastKey}
  onDismiss={() => (toastMessage = null)}
/>

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

  .actions button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .actions button.primary:not(:disabled):hover {
    filter: brightness(1.08);
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
