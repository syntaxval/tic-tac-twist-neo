use crate::board::{Board, MAX_PLAYERS};
use crate::rules::RuleSet;

pub const WIN_SCORE: i32 = 1_000_000;

/// Returns a cell index that immediately wins the game for `player`, if one exists.
#[inline]
pub fn find_immediate_win(board: &Board, rules: &RuleSet, player: u8) -> Option<u8> {
    let p_bits = board.player_bits(player);
    let others = board.occupied() & !p_bits;

    for win_mask in rules.masks() {
        if (others & win_mask.mask) != 0 {
            continue;
        }
        let owned = p_bits & win_mask.mask;
        if owned.count_ones() == u32::from(win_mask.len - 1) {
            let missing = win_mask.mask & !owned;
            return Some(missing.trailing_zeros() as u8);
        }
    }
    None
}

/// Checks if `ai_player` has an immediate win, or if any opponent (in turn order)
/// has an immediate 1-move win that must be blocked.
pub fn find_tactical_move(
    board: &Board,
    rules: &RuleSet,
    ai_player: u8,
    player_count: u8,
) -> Option<(u8, i32)> {
    if let Some(win_cell) = find_immediate_win(board, rules, ai_player) {
        return Some((win_cell, WIN_SCORE - 1));
    }

    for offset in 1..player_count {
        let opponent = (ai_player + offset) % player_count;
        if let Some(threat_cell) = find_immediate_win(board, rules, opponent) {
            return Some((threat_cell, 0));
        }
    }

    None
}

/// Returns the center cell index (optimal opening heuristic on empty boards).
#[inline]
pub fn center_cell(board: &Board) -> u8 {
    let mid = board.size() / 2;
    mid * board.size() + mid
}

/// Bitboard line-potential heuristic from `perspective`'s point of view.
pub fn evaluate_perspective(
    board: &Board,
    rules: &RuleSet,
    perspective: u8,
    next_turn: u8,
    player_count: u8,
) -> i32 {
    let occupied = board.occupied();
    let mut total = 0i32;

    for p in 0..player_count {
        let p_bits = board.player_bits(p);
        let others = occupied & !p_bits;
        let is_next = p == next_turn;
        let mut player_score = 0i32;

        for win_mask in rules.masks() {
            if (others & win_mask.mask) != 0 {
                continue;
            }
            let count = (p_bits & win_mask.mask).count_ones() as u8;
            if count == 0 {
                continue;
            }
            let remaining = win_mask.len.saturating_sub(count);
            let value = match remaining {
                0 => WIN_SCORE / 2,
                1 => {
                    if is_next {
                        6_000
                    } else {
                        1_600
                    }
                }
                2 => {
                    if is_next {
                        350
                    } else {
                        200
                    }
                }
                _ => i32::from(count) * i32::from(count) * 12,
            };
            player_score += value;
        }

        if p == perspective {
            total += player_score;
        } else {
            // Weight the immediate next opponent slightly higher.
            let penalty = if is_next {
                player_score + player_score / 4
            } else {
                player_score
            };
            total -= penalty;
        }
    }

    total
}

/// Evaluates a utility vector `[score_p0, score_p1, ...]` for MaxN.
pub fn evaluate_vector(
    board: &Board,
    rules: &RuleSet,
    next_turn: u8,
    player_count: u8,
) -> [i32; MAX_PLAYERS] {
    let mut scores = [0i32; MAX_PLAYERS];
    for p in 0..player_count {
        scores[p as usize] = evaluate_perspective(board, rules, p, next_turn, player_count);
    }
    scores
}

/// Orders legal moves for faster Alpha-Beta pruning:
/// 1. Transposition-table best move first
/// 2. Cells adjacent to occupied pieces + closer to board center
pub fn ordered_moves(board: &Board, tt_move: Option<u8>) -> Vec<u8> {
    let size = i32::from(board.size());
    let center = (size - 1) as f32 / 2.0;
    let occupied = board.occupied();

    let mut scored: Vec<(i32, u8)> = board
        .empty_cells()
        .map(|idx| {
            if tt_move == Some(idx) {
                return (100_000, idx);
            }

            let r = i32::from(idx / board.size());
            let c = i32::from(idx % board.size());

            // Distance from center (smaller is better).
            let dr = (r as f32 - center).abs();
            let dc = (c as f32 - center).abs();
            let center_bonus = 50 - ((dr + dc) * 5.0) as i32;

            // Bonus for being adjacent to an already occupied cell.
            let mut neighbor_bonus = 0;
            if occupied != 0 {
                for nr in (r - 1).max(0)..=(r + 1).min(size - 1) {
                    for nc in (c - 1).max(0)..=(c + 1).min(size - 1) {
                        if nr == r && nc == c {
                            continue;
                        }
                        let n_idx = (nr * size + nc) as u8;
                        if (occupied & (1u128 << n_idx)) != 0 {
                            neighbor_bonus += 25;
                        }
                    }
                }
            }

            (center_bonus + neighbor_bonus, idx)
        })
        .collect();

    scored.sort_unstable_by_key(|a| std::cmp::Reverse(a.0));
    scored.into_iter().map(|(_, idx)| idx).collect()
}
