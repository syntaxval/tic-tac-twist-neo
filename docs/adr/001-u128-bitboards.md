# ADR 001: `u128` bitboards for 3x3 to 10x10 boards

## Context

The game supports square boards from 3x3 (9 cells) to 10x10 (100 cells) with up to 4 players. The 2019 version stored the board as a string array and rebuilt 2D row, column, and diagonal matrices on every win check and coordinate lookup.

## Decision

Store each player's pieces in a single `u128` integer (`[u128; 4]`) alongside an `occupied: u128` mask. Precompute all winning lines and corner patterns as `u128` bitmasks (`WinMask`) when a game starts.

## Consequences

- Win checks reduce to `(player_bits & mask) == mask`. Threat scans use `count_ones()` and `trailing_zeros()`.
- Move make and unmake in AI search toggle two bitboards with zero heap allocations.
- Coordinate conversions (`coord_to_index` and `index_to_coord`) run in O(1) integer arithmetic.
- Board dimensions cap at 11x11 (121 bits fit in `u128`). Because the spec caps boards at 10x10 (100 bits), this avoids `BigInt` and heap-allocated bitsets.
