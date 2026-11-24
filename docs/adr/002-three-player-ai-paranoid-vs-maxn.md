# ADR 002: 3-Player AI — Paranoid Alpha-Beta vs. MaxN

## Context
Standard 2-player Minimax assumes a zero-sum alternating game. In a 3-player game, two opponents act between AI turns, and on a 5×5–10×10 board the branching factor makes unpruned full-depth search intractable.

## Decision
Provide a pluggable AI strategy layer with three algorithms sharing a fast bitboard tactical layer (immediate 1-move win → immediate 1-move opponent block):
1. **Paranoid Alpha-Beta (default):** Assumes all non-AI players cooperate against the AI, restoring classic Alpha-Beta pruning, paired with iterative deepening, a time budget, Zobrist hashing, and a 65,536-entry Transposition Table.
2. **MaxN:** Models each player maximizing their own component of a utility vector `[s_0, s_1, s_2]`. More faithful to self-interested multi-player play, at the cost of weaker pruning.
3. **Greedy (1-ply heuristic):** Fast baseline for testing and head-to-head visual benchmarks.

## Consequences
- Every search emits structured `SearchTelemetry` (depth, nodes, TT hits, elapsed ms, nodes/sec), enabling apples-to-apples comparisons.
- Terminal scores are ply-adjusted (`WIN_SCORE - ply`), ensuring faster wins and slower losses are always preferred, while draws score `0`.
