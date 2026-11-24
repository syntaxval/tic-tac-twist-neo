# ADR 002: Three-player AI with Paranoid Alpha-Beta and MaxN

## Context

Standard two-player Minimax assumes a zero-sum alternating game. In a three-player game, two opponents move between AI turns. On a 5x5 to 10x10 board, full-depth unpruned search exceeds interactive time limits.

## Decision

Run a fast bitboard tactical pass first (take a one-move win, or block any opponent's one-move threat in turn order), then dispatch to a selectable search algorithm:

1. Paranoid Alpha-Beta (default). Treats all non-AI players as minimizing against the AI, which restores Alpha-Beta pruning. Combines iterative deepening, a millisecond time budget, Zobrist hashing, and a 65,536-entry transposition table.
2. MaxN. Each player maximizes their own entry in a score vector `[s_0, s_1, s_2]`. This models self-interested multi-player turns more accurately than Paranoid, at the cost of weaker pruning.
3. Greedy (1-ply heuristic). Evaluates immediate line potential for fast baseline testing and comparisons.

## Consequences

- Every search returns a `SearchTelemetry` record with algorithm, depth reached, nodes evaluated, transposition table hits, elapsed milliseconds, and nodes per second.
- Terminal scores adjust by ply (`WIN_SCORE - ply`), so the engine prefers faster wins and slower losses while scoring draws at `0`.
