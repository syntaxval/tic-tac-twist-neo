use web_time::{Duration, Instant};

use crate::ai::heuristic::{
    WIN_SCORE, center_cell, evaluate_perspective, find_tactical_move, ordered_moves,
};
use crate::ai::zobrist::{Bound, TranspositionTable, ZobristTable};
use crate::ai::{AiAlgorithm, AiConfig, SearchResult, SearchTelemetry};
use crate::board::Board;
use crate::error::EngineError;
use crate::rules::RuleSet;

struct SearchContext<'a> {
    rules: &'a RuleSet,
    ai_player: u8,
    player_count: u8,
    zobrist: ZobristTable,
    tt: TranspositionTable,
    deadline: Instant,
    nodes: u64,
    tt_hits: u64,
    timed_out: bool,
}

impl<'a> SearchContext<'a> {
    #[inline]
    fn check_timeout(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        // Check clock every 1024 nodes to avoid syscall overhead in the hot loop.
        if (self.nodes & 1023) == 0 && Instant::now() >= self.deadline {
            self.timed_out = true;
            return true;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn alphabeta(
        &mut self,
        board: &mut Board,
        hash: u64,
        depth: u8,
        ply: u8,
        mut alpha: i32,
        mut beta: i32,
        turn: u8,
        last_player: u8,
    ) -> i32 {
        self.nodes += 1;
        if self.check_timeout() {
            return 0;
        }

        // Terminal win check for the player who just moved.
        if self.rules.has_won(board, last_player) {
            return if last_player == self.ai_player {
                WIN_SCORE - i32::from(ply)
            } else {
                -WIN_SCORE + i32::from(ply)
            };
        }

        if board.is_full() {
            return 0;
        }

        if depth == 0 {
            return evaluate_perspective(board, self.rules, self.ai_player, turn, self.player_count);
        }

        let orig_alpha = alpha;
        let orig_beta = beta;
        let mut tt_move = None;

        if let Some(entry) = self.tt.get(hash) {
            tt_move = Some(entry.best_move);
            if entry.depth >= depth {
                self.tt_hits += 1;
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        let moves = ordered_moves(board, tt_move);
        let maximizing = turn == self.ai_player;
        let next_turn = (turn + 1) % self.player_count;
        let turn_xor = self.zobrist.turn_key(turn) ^ self.zobrist.turn_key(next_turn);

        let mut best_score = if maximizing { i32::MIN + 1 } else { i32::MAX };
        let mut best_move = moves[0];

        for mv in moves {
            board.place_unchecked(turn, mv);
            let next_hash = hash ^ self.zobrist.cell_key(turn, mv) ^ turn_xor;

            let score = self.alphabeta(
                board,
                next_hash,
                depth - 1,
                ply + 1,
                alpha,
                beta,
                next_turn,
                turn,
            );

            board.undo(turn, mv);

            if self.timed_out {
                return 0;
            }

            if maximizing {
                if score > best_score {
                    best_score = score;
                    best_move = mv;
                }
                alpha = alpha.max(best_score);
            } else {
                if score < best_score {
                    best_score = score;
                    best_move = mv;
                }
                beta = beta.min(best_score);
            }

            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= orig_alpha {
            Bound::Upper
        } else if best_score >= orig_beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.tt.store(hash, depth, best_score, bound, best_move);
        best_score
    }
}

/// Paranoid Alpha-Beta search with iterative deepening and Zobrist transposition table.
pub fn search_paranoid(
    board: &Board,
    rules: &RuleSet,
    ai_player: u8,
    player_count: u8,
    config: &AiConfig,
) -> Result<SearchResult, EngineError> {
    if board.is_full() {
        return Err(EngineError::NoLegalMoves);
    }

    let start = Instant::now();

    if board.is_empty() {
        return Ok(SearchResult {
            best_move: center_cell(board),
            score: 0,
            telemetry: SearchTelemetry::instant(AiAlgorithm::Paranoid, 1, 1, start.elapsed()),
        });
    }

    if let Some((mv, score)) = find_tactical_move(board, rules, ai_player, player_count) {
        return Ok(SearchResult {
            best_move: mv,
            score,
            telemetry: SearchTelemetry::instant(AiAlgorithm::Paranoid, 1, 1, start.elapsed()),
        });
    }

    let budget = Duration::from_millis(config.time_budget_ms.max(5));
    let mut ctx = SearchContext {
        rules,
        ai_player,
        player_count,
        zobrist: ZobristTable::new(),
        tt: TranspositionTable::new(),
        deadline: start + budget,
        nodes: 0,
        tt_hits: 0,
        timed_out: false,
    };

    let mut work_board = board.clone();
    let root_hash = ctx.zobrist.hash_board(&work_board, player_count, ai_player);
    let next_turn = (ai_player + 1) % player_count;
    let turn_xor = ctx.zobrist.turn_key(ai_player) ^ ctx.zobrist.turn_key(next_turn);

    let max_depth = config.max_depth.clamp(1, board.empty_count().min(64) as u8);
    let mut best_move = ordered_moves(&work_board, None)[0];
    let mut best_score = 0;
    let mut depth_reached = 0u8;

    for depth in 1..=max_depth {
        let tt_move = ctx.tt.get(root_hash).map(|e| e.best_move).or(Some(best_move));
        let moves = ordered_moves(&work_board, tt_move);

        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;
        let mut iter_best_move = moves[0];
        let mut iter_best_score = i32::MIN + 1;

        for mv in moves {
            work_board.place_unchecked(ai_player, mv);
            let next_hash = root_hash ^ ctx.zobrist.cell_key(ai_player, mv) ^ turn_xor;

            let score = ctx.alphabeta(
                &mut work_board,
                next_hash,
                depth - 1,
                1,
                alpha,
                beta,
                next_turn,
                ai_player,
            );

            work_board.undo(ai_player, mv);

            if ctx.timed_out && depth > 1 {
                break;
            }

            if score > iter_best_score {
                iter_best_score = score;
                iter_best_move = mv;
            }
            alpha = alpha.max(iter_best_score);
        }

        if ctx.timed_out && depth > 1 {
            break;
        }

        best_move = iter_best_move;
        best_score = iter_best_score;
        depth_reached = depth;
        ctx.tt
            .store(root_hash, depth, best_score, Bound::Exact, best_move);

        // Stop early if a forced win is already found.
        if best_score >= WIN_SCORE - 100 {
            break;
        }
    }

    Ok(SearchResult {
        best_move,
        score: best_score,
        telemetry: SearchTelemetry::new(
            AiAlgorithm::Paranoid,
            depth_reached.max(1),
            ctx.nodes,
            ctx.tt_hits,
            start.elapsed(),
        ),
    })
}
