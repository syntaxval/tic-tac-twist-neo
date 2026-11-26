<script lang="ts">
  import type {
    AiAlgorithm,
    Outcome,
    PlayerConfig,
    PlayerTelemetryState,
  } from '../types';

  interface Props {
    players: PlayerConfig[];
    playerStats: PlayerTelemetryState[];
    currentTurn: number;
    thinking: boolean;
    outcome: Outcome;
    resetKey: number;
  }

  let { players, playerStats, currentTurn, thinking, outcome, resetKey }: Props = $props();

  type TabId = number | 'tally';

  function defaultTab(list: PlayerConfig[]): number {
    const firstAi = list.findIndex((p) => p.kind.type === 'ai');
    return firstAi >= 0 ? firstAi : 0;
  }

  let selectedTab = $state<TabId>(0);
  let showGuide = $state(false);
  const exactFmt = new Intl.NumberFormat();

  // Only reset selected tab when a brand new config/match is initialized (never auto-switch mid-game).
  $effect(() => {
    void resetKey;
    selectedTab = defaultTab(players);
  });

  const isGameOver = $derived(outcome.status !== 'ongoing');

  function formatCompact(value: number): string {
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
    if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
    return String(value);
  }

  function formatMs(ms: number): string {
    if (ms >= 10_000) return `${(ms / 1000).toFixed(1)}s`;
    if (ms >= 1000) return `${(ms / 1000).toFixed(2)}s`;
    return `${ms.toFixed(1)}ms`;
  }

  function formatBudgetShort(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(2).replace(/\.?0+$/, '')}s` : `${ms}ms`;
  }

  function algoLabel(algo: AiAlgorithm): string {
    if (algo === 'paranoid') return 'Paranoid';
    if (algo === 'maxN') return 'MaxN';
    return 'Greedy';
  }

  function buildSparkline(values: number[], useLog: boolean) {
    const W = 132;
    const H = 36;
    const padX = 4;
    const padY = 5;

    if (values.length === 0) {
      return { line: '', area: '', lastX: 0, lastY: 0, count: 0 };
    }

    const mapped = values.map((v) => (useLog ? Math.log10(Math.max(1, v)) : Math.max(0, v)));
    const maxVal = Math.max(useLog ? 2 : 100, ...mapped);
    const stepX = mapped.length > 1 ? (W - padX * 2) / (mapped.length - 1) : 0;

    const pts = mapped.map((v, idx) => {
      const x = mapped.length === 1 ? W - padX : padX + idx * stepX;
      const y = H - padY - (v / maxVal) * (H - padY * 2);
      return { x, y };
    });

    const line = pts
      .map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x.toFixed(1)},${p.y.toFixed(1)}`)
      .join(' ');
    const first = pts[0];
    const last = pts[pts.length - 1];
    const area =
      pts.length > 1 ? `${line} L${last.x.toFixed(1)},${H} L${first.x.toFixed(1)},${H} Z` : '';

    return { line, area, lastX: last.x, lastY: last.y, count: pts.length };
  }

  // Semicircle arc geometry
  const ARC_RADIUS = 34;
  const ARC_LENGTH = Math.PI * ARC_RADIUS;

  const activePlayerIndex = $derived(typeof selectedTab === 'number' ? selectedTab : 0);
  const activePlayer = $derived(players[activePlayerIndex]);
  const activeStats = $derived(playerStats[activePlayerIndex]);

  // --- AI Derived Metrics ---
  const latestAi = $derived(
    activeStats && activeStats.aiHistory.length > 0
      ? activeStats.aiHistory[activeStats.aiHistory.length - 1]
      : null
  );
  const aiCfg = $derived(activePlayer?.kind.type === 'ai' ? activePlayer.kind : null);
  const isGreedy = $derived(aiCfg?.algorithm === 'greedy');

  const speedRatio = $derived.by(() => {
    if (!latestAi || latestAi.nodesPerSec <= 0) return 0;
    const logMin = Math.log10(1_000);
    const logMax = Math.log10(2_000_000);
    const ratio = (Math.log10(Math.max(1_000, latestAi.nodesPerSec)) - logMin) / (logMax - logMin);
    return Math.min(1, Math.max(0.06, ratio));
  });

  const speedColor = $derived.by(() => {
    if (!latestAi) return 'var(--border)';
    const hue = Math.round(215 - speedRatio * 67);
    const sat = Math.round(45 + speedRatio * 35);
    return `hsl(${hue} ${sat}% 48%)`;
  });

  const depthRatio = $derived.by(() => {
    if (!latestAi) return 0;
    return Math.min(1, Math.max(0.1, latestAi.depthReached / 10));
  });

  const depthColor = $derived.by(() => {
    if (!latestAi) return 'var(--border)';
    const hue = Math.round(210 - depthRatio * 65);
    const sat = Math.round(50 + depthRatio * 30);
    return `hsl(${hue} ${sat}% 48%)`;
  });

  const budgetRatio = $derived.by(() => {
    if (!latestAi || !aiCfg) return 0;
    if (isGreedy) return 0.04;
    if (aiCfg.timeBudgetMs <= 0) return 0;
    return Math.min(1, Math.max(0.03, latestAi.elapsedMs / aiCfg.timeBudgetMs));
  });

  const budgetColor = $derived.by(() => {
    if (!latestAi) return 'var(--border)';
    const hue = Math.round(148 - budgetRatio * 108);
    return `hsl(${hue} 76% 46%)`;
  });

  const cacheRate = $derived.by(() => {
    if (!latestAi || latestAi.nodesEvaluated === 0) return 0;
    return Math.min(100, Math.round((latestAi.ttHits / latestAi.nodesEvaluated) * 100));
  });

  const aiSpark = $derived(
    buildSparkline(
      (activeStats?.aiHistory ?? []).map((h) => h.nodesEvaluated),
      true
    )
  );

  // --- Human Derived Metrics ---
  const humanMoves = $derived(activeStats?.humanMoveTimesMs ?? []);
  const lastHumanMs = $derived(humanMoves.length > 0 ? humanMoves[humanMoves.length - 1] : null);
  const avgHumanMs = $derived(
    activeStats && activeStats.turnCount > 0
      ? activeStats.totalTimeMs / activeStats.turnCount
      : null
  );

  // Faster human pace (< 1s) fills gauge toward bright green; slower (> 10s) stays low/amber
  const paceRatio = $derived.by(() => {
    if (avgHumanMs === null) return 0;
    const clamped = Math.min(12_000, Math.max(400, avgHumanMs));
    return Math.min(1, Math.max(0.1, 1 - (clamped - 400) / (12_000 - 400)));
  });

  const paceColor = $derived.by(() => {
    if (avgHumanMs === null) return 'var(--border)';
    const hue = Math.round(35 + paceRatio * 115); // amber 35 -> emerald 150
    return `hsl(${hue} 74% 48%)`;
  });

  const humanSpark = $derived(buildSparkline(humanMoves, false));
</script>

<section class="hud" aria-label="Player telemetry dashboard">
  <header class="hud-head">
    <div class="tabs" role="tablist" aria-label="Select player telemetry">
      {#each players as p, idx}
        {@const isTurn = !isGameOver && currentTurn === idx}
        <button
          type="button"
          role="tab"
          aria-selected={selectedTab === idx}
          class="tab"
          class:selected={selectedTab === idx}
          onclick={() => (selectedTab = idx)}
        >
          <span
            class="turn-dot"
            class:live={isTurn}
            class:pulsing={isTurn && thinking}
            style:--dot-color={p.color}
          ></span>
          <span class="tab-sym" style:color={p.color}>{p.symbol}</span>
          <span class="tab-name">{p.name}</span>
          <span class="tab-tag">
            {p.kind.type === 'ai' ? algoLabel(p.kind.algorithm) : 'Human'}
          </span>
        </button>
      {/each}

      {#if isGameOver}
        <button
          type="button"
          role="tab"
          aria-selected={selectedTab === 'tally'}
          class="tab tally-tab"
          class:selected={selectedTab === 'tally'}
          onclick={() => (selectedTab = 'tally')}
        >
          Match Tally
        </button>
      {/if}
    </div>

    {#if selectedTab !== 'tally'}
      <button
        type="button"
        class="guide-toggle"
        aria-expanded={showGuide}
        aria-controls="telemetry-guide"
        onclick={() => (showGuide = !showGuide)}
      >
        {showGuide ? 'Hide' : '?'}
      </button>
    {/if}
  </header>

  {#if selectedTab === 'tally'}
    <div class="tally" role="tabpanel" aria-label="Match tally">
      <table>
        <thead>
          <tr>
            <th>Player</th>
            <th>Mode</th>
            <th>Turns</th>
            <th>Total / Avg</th>
            <th>AI Detail</th>
          </tr>
        </thead>
        <tbody>
          {#each players as p, idx}
            {@const st = playerStats[idx]}
            {@const isWinner = outcome.status === 'win' && outcome.player === idx}
            {@const avgMs = st.turnCount > 0 ? st.totalTimeMs / st.turnCount : 0}
            {@const avgDepth = st.turnCount > 0 ? (st.depthSum / st.turnCount).toFixed(1) : '-'}
            <tr>
              <td class="player-cell">
                <span style:color={p.color}>{p.symbol}</span>
                <strong>{p.name}</strong>
                {#if isWinner}
                  <span class="win-pill">Win</span>
                {/if}
              </td>
              <td>{p.kind.type === 'ai' ? algoLabel(p.kind.algorithm) : 'Human'}</td>
              <td class="num">{st.turnCount}</td>
              <td class="num">
                {st.turnCount > 0 ? `${formatMs(st.totalTimeMs)} (${formatMs(avgMs)})` : '-'}
              </td>
              <td class="num">
                {#if p.kind.type === 'ai' && st.turnCount > 0}
                  {formatCompact(st.totalNodes)} pos · d{avgDepth} · {formatCompact(st.peakNodesPerSec)}/s
                {:else if p.kind.type === 'human' && st.fastestHumanMs !== null}
                  fastest {formatMs(st.fastestHumanMs)}
                {:else}
                  -
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if activePlayer?.kind.type === 'ai'}
    <!-- AI Instrument View -->
    <div class="instruments" role="tabpanel" aria-label={`AI telemetry for ${activePlayer.name}`}>
      <div
        class="gauge-card"
        title={latestAi
          ? `${exactFmt.format(latestAi.nodesPerSec)} board positions checked per second`
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
            {latestAi ? `${formatCompact(latestAi.nodesPerSec)}/s` : '-'}
          </span>
          <span class="gauge-label">Speed</span>
        </div>
      </div>

      <div class="bars-card">
        <div class="bar-row">
          <div class="bar-meta">
            <span class="bar-label">Lookahead</span>
            <span class="bar-val">
              {latestAi ? `${latestAi.depthReached} ${latestAi.depthReached === 1 ? 'ply' : 'plies'}` : '-'}
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
                {latestAi ? `${formatMs(latestAi.elapsedMs)} (1-ply)` : '1-ply'}
              {:else if aiCfg}
                {latestAi
                  ? `${formatMs(latestAi.elapsedMs)} / ${formatBudgetShort(aiCfg.timeBudgetMs)}`
                  : `- / ${formatBudgetShort(aiCfg.timeBudgetMs)}`}
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
            title={latestAi
              ? `${exactFmt.format(latestAi.ttHits)} repeated positions served from cache (${cacheRate}%)`
              : 'Cached positions reused during search'}
          >
            Cache hits <strong>{latestAi ? formatCompact(latestAi.ttHits) : '-'}</strong>
          </span>
          <span>
            Turns <strong>{activeStats?.turnCount ?? 0}</strong>
          </span>
        </div>
      </div>

      <div
        class="spark-card"
        title={latestAi
          ? `${exactFmt.format(latestAi.nodesEvaluated)} positions evaluated on last AI turn`
          : 'Positions evaluated per AI turn'}
      >
        <div class="spark-meta">
          <span class="bar-label">Positions</span>
          <span class="bar-val">
            {latestAi ? formatCompact(latestAi.nodesEvaluated) : '-'}
          </span>
        </div>
        <svg class="spark-svg" viewBox="0 0 132 36" preserveAspectRatio="none" aria-hidden="true">
          <defs>
            <linearGradient id="aiSparkFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#22c55e" stop-opacity="0.32" />
              <stop offset="100%" stop-color="#22c55e" stop-opacity="0.02" />
            </linearGradient>
          </defs>
          <line x1="0" y1="35" x2="132" y2="35" class="spark-baseline" />
          {#if aiSpark.area}
            <path d={aiSpark.area} fill="url(#aiSparkFill)" />
          {/if}
          {#if aiSpark.line}
            <path
              d={aiSpark.line}
              fill="none"
              stroke="#22c55e"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle cx={aiSpark.lastX} cy={aiSpark.lastY} r="2.6" fill="#22c55e" />
          {/if}
        </svg>
        <span class="spark-caption">
          {aiSpark.count > 0 ? `Last ${aiSpark.count} ${aiSpark.count === 1 ? 'move' : 'moves'}` : 'Waiting for move'}
        </span>
      </div>
    </div>
  {:else if activePlayer}
    <!-- Human Instrument View -->
    <div class="instruments" role="tabpanel" aria-label={`Human timing for ${activePlayer.name}`}>
      <div
        class="gauge-card"
        title={avgHumanMs !== null
          ? `Average human think time: ${formatMs(avgHumanMs)}`
          : 'Human pace gauge'}
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
            stroke={paceColor}
            stroke-width="6.5"
            stroke-linecap="round"
            stroke-dasharray={ARC_LENGTH}
            stroke-dashoffset={ARC_LENGTH * (1 - paceRatio)}
          />
        </svg>
        <div class="gauge-readout">
          <span class="gauge-val">
            {avgHumanMs !== null ? formatMs(avgHumanMs) : '-'}
          </span>
          <span class="gauge-label">Avg pace</span>
        </div>
      </div>

      <div class="bars-card">
        <div class="bar-row">
          <div class="bar-meta">
            <span class="bar-label">Last move</span>
            <span class="bar-val">
              {lastHumanMs !== null ? formatMs(lastHumanMs) : '-'}
            </span>
          </div>
          <div class="track">
            <div
              class="fill"
              style:width={lastHumanMs !== null
                ? `${Math.min(100, Math.max(6, (lastHumanMs / 10_000) * 100))}%`
                : '0%'}
              style:background={activePlayer.color}
            ></div>
          </div>
        </div>

        <div class="bar-row">
          <div class="bar-meta">
            <span class="bar-label">Fastest move</span>
            <span class="bar-val">
              {activeStats?.fastestHumanMs !== null && activeStats?.fastestHumanMs !== undefined
                ? formatMs(activeStats.fastestHumanMs)
                : '-'}
            </span>
          </div>
          <div class="track">
            <div
              class="fill"
              style:width={activeStats?.fastestHumanMs !== null && activeStats?.fastestHumanMs !== undefined
                ? `${Math.min(100, Math.max(6, (activeStats.fastestHumanMs / 10_000) * 100))}%`
                : '0%'}
              style:background="#22c55e"
            ></div>
          </div>
        </div>

        <div class="sub-stats">
          <span>
            Total <strong>{activeStats && activeStats.turnCount > 0 ? formatMs(activeStats.totalTimeMs) : '-'}</strong>
          </span>
          <span>
            Turns <strong>{activeStats?.turnCount ?? 0}</strong>
          </span>
        </div>
      </div>

      <div class="spark-card">
        <div class="spark-meta">
          <span class="bar-label">Move times</span>
          <span class="bar-val">
            {lastHumanMs !== null ? formatMs(lastHumanMs) : '-'}
          </span>
        </div>
        <svg class="spark-svg" viewBox="0 0 132 36" preserveAspectRatio="none" aria-hidden="true">
          <defs>
            <linearGradient id="humanSparkFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color={activePlayer.color} stop-opacity="0.32" />
              <stop offset="100%" stop-color={activePlayer.color} stop-opacity="0.02" />
            </linearGradient>
          </defs>
          <line x1="0" y1="35" x2="132" y2="35" class="spark-baseline" />
          {#if humanSpark.area}
            <path d={humanSpark.area} fill="url(#humanSparkFill)" />
          {/if}
          {#if humanSpark.line}
            <path
              d={humanSpark.line}
              fill="none"
              stroke={activePlayer.color}
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle cx={humanSpark.lastX} cy={humanSpark.lastY} r="2.6" fill={activePlayer.color} />
          {/if}
        </svg>
        <span class="spark-caption">
          {humanSpark.count > 0 ? `Last ${humanSpark.count} ${humanSpark.count === 1 ? 'move' : 'moves'}` : 'Waiting for move'}
        </span>
      </div>
    </div>
  {/if}

  {#if showGuide && selectedTab !== 'tally'}
    {#if activePlayer?.kind.type === 'ai'}
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
          <dd>Milliseconds spent out of this player's time budget. Instant tactical wins or blocks finish in under 1ms.</dd>
        </div>
        <div>
          <dt>Positions</dt>
          <dd>Total future board states evaluated on the last move, plotted across recent turns.</dd>
        </div>
        <div>
          <dt>Cache hits</dt>
          <dd>Repeated board states reused from the Zobrist transposition table instead of re-searching.</dd>
        </div>
      </dl>
    {:else}
      <dl id="telemetry-guide" class="guide">
        <div>
          <dt>Avg pace</dt>
          <dd>Average time taken per move. Greener means snappier decisions.</dd>
        </div>
        <div>
          <dt>Move times</dt>
          <dd>Elapsed time from turn start to click, plotted across recent moves.</dd>
        </div>
      </dl>
    {/if}
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

  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    min-width: 0;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.22rem 0.5rem;
    border-radius: 0.45rem;
    border: 1px solid transparent;
    background: transparent;
    color: var(--muted);
    font-size: 0.74rem;
    white-space: nowrap;
  }

  .tab:hover {
    background: var(--surface-alt);
    color: var(--text);
  }

  .tab.selected {
    border-color: var(--border);
    background: var(--surface-alt);
    color: var(--text);
  }

  .tally-tab {
    font-weight: 600;
    color: var(--accent);
  }

  .turn-dot {
    width: 0.42rem;
    height: 0.42rem;
    border-radius: 999px;
    background: transparent;
    flex-shrink: 0;
  }

  .turn-dot.live {
    background: var(--dot-color);
  }

  .turn-dot.pulsing {
    animation: pulse 900ms ease-in-out infinite;
  }

  .tab-sym {
    font-weight: 700;
  }

  .tab-name {
    font-weight: 600;
  }

  .tab-tag {
    font-size: 0.66rem;
    color: var(--muted);
  }

  .guide-toggle {
    width: 1.5rem;
    height: 1.5rem;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface-alt);
    color: var(--muted);
    font-size: 0.72rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  .guide-toggle:hover {
    color: var(--text);
    border-color: var(--accent);
  }

  .instruments {
    display: grid;
    grid-template-columns: 108px minmax(0, 1fr) 136px;
    gap: 0.8rem;
    align-items: center;
  }

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

  /* Match Tally table */
  .tally {
    overflow-x: auto;
  }

  .tally table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.74rem;
  }

  .tally th {
    text-align: left;
    font-weight: 600;
    color: var(--muted);
    padding: 0.3rem 0.45rem;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  .tally td {
    padding: 0.4rem 0.45rem;
    color: var(--text);
    border-bottom: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
    white-space: nowrap;
  }

  .tally tr:last-child td {
    border-bottom: none;
  }

  .player-cell {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .win-pill {
    padding: 0.05rem 0.35rem;
    border-radius: 999px;
    background: color-mix(in srgb, #22c55e 20%, transparent);
    color: #22c55e;
    font-size: 0.66rem;
    font-weight: 700;
  }

  .num {
    font-variant-numeric: tabular-nums;
  }

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
    grid-template-columns: 96px minmax(0, 1fr);
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
    .tab-tag {
      display: none;
    }

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
    .turn-dot {
      transition: none;
      animation: none;
    }
  }
</style>
