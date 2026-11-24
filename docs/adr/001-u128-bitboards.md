# ADR 001: `u128` Bitboards for 3×3 through 10×10 Boards

## Context
The game supports square boards from 3×3 (9 cells) up to 10×10 (100 cells) with up to 4 players. The 2019 implementation stored board state as an array of strings and rebuilt 2D row/column/diagonal matrices on every win check and coordinate conversion.

## Decision
Represent each player's pieces as a single `u128` bitboard (`[u128; 4]`) plus an `occupied: u128` mask, and precompute all winning patterns as `u128` bitmasks (`WinMask`).

## Consequences
- **Speed:** Win checks become `(player_bits & mask) == mask`, immediate-threat detection uses `count_ones()` and `trailing_zeros()`, and move make/unmake in AI search is two bitwise instructions with zero heap allocation.
- **Coordinate conversion:** `coord_to_index` and `index_to_coord` are $O(1)$ integer arithmetic.
- **Trade-off:** Board size is capped at 11×11 (121 bits ≤ 128). Since the spec caps boards at 10×10 (100 bits), this fits natively without `BigInt` or heap-allocated bitsets.
