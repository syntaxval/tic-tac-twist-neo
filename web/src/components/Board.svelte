<script lang="ts">
  import type { GameSnapshot } from '../types';

  interface Props {
    snapshot: GameSnapshot;
    disabled: boolean;
    onPlay: (cell: number) => void;
  }

  let { snapshot, disabled, onPlay }: Props = $props();

  let gridEl: HTMLDivElement | undefined = $state();
  let focusedIndex = $state(0);

  const winningSet = $derived(
    snapshot.outcome.status === 'win'
      ? new Set(snapshot.outcome.cells)
      : new Set<number>()
  );

  const lastMove = $derived(
    snapshot.history.length > 0
      ? snapshot.history[snapshot.history.length - 1].cell
      : -1
  );

  const winLineCoords = $derived.by(() => {
    if (
      snapshot.outcome.status !== 'win' ||
      snapshot.outcome.kind === 'fourCorners' ||
      snapshot.outcome.cells.length < 2
    ) {
      return null;
    }
    const n = snapshot.boardSize;
    const first = snapshot.outcome.cells[0];
    const last = snapshot.outcome.cells[snapshot.outcome.cells.length - 1];
    return {
      x1: ((first % n) + 0.5) * (100 / n),
      y1: (Math.floor(first / n) + 0.5) * (100 / n),
      x2: ((last % n) + 0.5) * (100 / n),
      y2: (Math.floor(last / n) + 0.5) * (100 / n),
      color: snapshot.players[snapshot.outcome.player]?.color ?? 'currentColor',
    };
  });

  // Reset roving focus when board size changes.
  $effect(() => {
    if (focusedIndex >= snapshot.cells.length) {
      focusedIndex = 0;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (
      e.defaultPrevented ||
      e.altKey ||
      e.ctrlKey ||
      e.metaKey ||
      e.shiftKey ||
      !gridEl
    ) {
      return;
    }

    const dir =
      e.key === 'ArrowRight'
        ? 'right'
        : e.key === 'ArrowLeft'
          ? 'left'
          : e.key === 'ArrowDown'
            ? 'down'
            : e.key === 'ArrowUp'
              ? 'up'
              : null;

    if (!dir) return;

    const target = e.composedPath()[0];
    if (!(target instanceof HTMLButtonElement) || !gridEl.contains(target)) {
      return;
    }

    const candidates = Array.from(
      gridEl.querySelectorAll<HTMLButtonElement>('button[data-cell]')
    ).filter((el) => el.checkVisibility());

    if (!candidates.includes(target)) return;

    const currentRect = target.getBoundingClientRect();
    const currentCx = currentRect.left + currentRect.width / 2;
    const currentCy = currentRect.top + currentRect.height / 2;
    const tol = 2;

    let bestAligned: { el: HTMLButtonElement; gap: number; perp: number } | null = null;
    let bestFallback: { el: HTMLButtonElement; score: number } | null = null;

    for (const el of candidates) {
      if (el === target) continue;
      const r = el.getBoundingClientRect();
      const cx = r.left + r.width / 2;
      const cy = r.top + r.height / 2;

      let inDir = false;
      let gap = 0;
      let overlaps = false;
      let perp = 0;

      if (dir === 'right' && r.left >= currentRect.right - tol) {
        inDir = true;
        gap = Math.max(0, r.left - currentRect.right);
        overlaps = r.bottom > currentRect.top && r.top < currentRect.bottom;
        perp = Math.abs(cy - currentCy);
      } else if (dir === 'left' && r.right <= currentRect.left + tol) {
        inDir = true;
        gap = Math.max(0, currentRect.left - r.right);
        overlaps = r.bottom > currentRect.top && r.top < currentRect.bottom;
        perp = Math.abs(cy - currentCy);
      } else if (dir === 'down' && r.top >= currentRect.bottom - tol) {
        inDir = true;
        gap = Math.max(0, r.top - currentRect.bottom);
        overlaps = r.right > currentRect.left && r.left < currentRect.right;
        perp = Math.abs(cx - currentCx);
      } else if (dir === 'up' && r.bottom <= currentRect.top + tol) {
        inDir = true;
        gap = Math.max(0, currentRect.top - r.bottom);
        overlaps = r.right > currentRect.left && r.left < currentRect.right;
        perp = Math.abs(cx - currentCx);
      }

      if (!inDir) continue;

      if (overlaps) {
        if (
          !bestAligned ||
          gap < bestAligned.gap ||
          (gap === bestAligned.gap && perp < bestAligned.perp)
        ) {
          bestAligned = { el, gap, perp };
        }
      } else {
        const score = gap + 2 * perp;
        if (!bestFallback || score < bestFallback.score) {
          bestFallback = { el, score };
        }
      }
    }

    const next = bestAligned?.el ?? bestFallback?.el;
    if (!next) return; // Preserve native viewport scrolling at grid boundaries.

    e.preventDefault();
    const nextIdx = Number(next.dataset.cell);
    focusedIndex = nextIdx;
    next.focus();
    next.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }
</script>

<div class="board-shell">
  <div
    bind:this={gridEl}
    class="board-grid"
    role="grid"
    tabindex="-1"
    aria-label="Tic-Tac-Twist {snapshot.boardSize} by {snapshot.boardSize} board"
    style:--size={snapshot.boardSize}
    onkeydown={handleKeydown}
  >
    {#each snapshot.cells as owner, idx (idx)}
      {@const isOccupied = owner != null}
      {@const player = isOccupied ? snapshot.players[owner] : null}
      {@const isWinCell = winningSet.has(idx)}
      {@const isLast = idx === lastMove}
      {@const row = Math.floor(idx / snapshot.boardSize) + 1}
      {@const col = (idx % snapshot.boardSize) + 1}
      <button
        type="button"
        role="gridcell"
        data-cell={idx}
        tabindex={focusedIndex === idx ? 0 : -1}
        class="cell"
        class:occupied={isOccupied}
        class:win-cell={isWinCell}
        class:last-move={isLast}
        disabled={disabled || isOccupied || snapshot.outcome.status !== 'ongoing'}
        aria-label={`Row ${row}, Column ${col}: ${player ? player.name : 'empty'}`}
        onclick={() => onPlay(idx)}
        onfocus={() => (focusedIndex = idx)}
        style:--player-color={player?.color ?? 'transparent'}
      >
        {#if player}
          <span class="symbol">{player.symbol}</span>
        {/if}
      </button>
    {/each}

    {#if winLineCoords}
      <svg class="win-overlay" viewBox="0 0 100 100" aria-hidden="true">
        <line
          x1={winLineCoords.x1}
          y1={winLineCoords.y1}
          x2={winLineCoords.x2}
          y2={winLineCoords.y2}
          stroke={winLineCoords.color}
          stroke-width={Math.max(1.4, 4.5 / snapshot.boardSize * 1.4)}
          stroke-linecap="round"
        />
      </svg>
    {/if}
  </div>
</div>

<style>
  .board-shell {
    width: min(100%, 540px);
    aspect-ratio: 1;
    margin-inline: auto;
  }

  .board-grid {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--size), minmax(0, 1fr));
    grid-template-rows: repeat(var(--size), minmax(0, 1fr));
    gap: clamp(3px, calc(18px / var(--size)), 8px);
    width: 100%;
    height: 100%;
    padding: 0.75rem;
    border-radius: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: 0 12px 32px rgb(0 0 0 / 0.12);
  }

  .cell {
    display: grid;
    place-items: center;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: clamp(4px, calc(24px / var(--size)), 12px);
    background: var(--surface-alt);
    transition: transform 120ms ease, border-color 120ms ease, background-color 120ms ease;
    user-select: none;
    touch-action: manipulation;
  }

  .cell:not(:disabled):hover {
    border-color: var(--accent);
    transform: translateY(-1px);
  }

  .cell:disabled {
    cursor: default;
  }

  .cell.last-move {
    border-color: var(--player-color);
    box-shadow: inset 0 0 0 1px var(--player-color);
  }

  .cell.win-cell {
    border-color: var(--player-color);
    background: color-mix(in srgb, var(--player-color) 18%, var(--surface-alt));
  }

  .symbol {
    font-size: clamp(0.85rem, calc(16rem / var(--size) * 0.48), 2.8rem);
    line-height: 1;
    color: var(--player-color);
    animation: pop 140ms cubic-bezier(0.2, 0.9, 0.3, 1.2);
  }

  .win-overlay {
    position: absolute;
    inset: 0.75rem;
    width: calc(100% - 1.5rem);
    height: calc(100% - 1.5rem);
    pointer-events: none;
  }

  .win-overlay line {
    opacity: 0.85;
    stroke-dasharray: 200;
    stroke-dashoffset: 0;
    animation: draw 280ms ease-out;
  }

  @keyframes pop {
    from {
      transform: scale(0.45);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
  }

  @keyframes draw {
    from {
      stroke-dashoffset: 200;
    }
    to {
      stroke-dashoffset: 0;
    }
  }
</style>
