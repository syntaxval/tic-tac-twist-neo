use web_time::{Duration, Instant};

use crate::ai::heuristic::{
    WIN_SCORE, center_cell, evaluate_vector, find_tactical_move, ordered_moves,
};
use crate::ai::{AiAlgorithm, AiConfig, SearchResult, SearchTelemetry};
use crate::board::{Board, MAX_PLAYERS};
use crate::error::EngineError;
use crate::rules::RuleSet;

struct MaxNContext<'a> {
    rules: &'a RuleSet,
    player_count: u8,
    deadline: Instant,
    nodes: u64,
    timed_out: bool,
}

impl<'a> MaxNContext<'a> {
    #[inline]
    fn check_timeout(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        if (self.nodes & 1023) == 0 && Instant::now() >= self.deadline {
            self.timed_out = true;
            return true;
        }
        false
    }

    fn maxn(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: u8,
        turn: u8,
        last_player: u8,
    ) -> [i32; MAX_PLAYERS] {
        self.nodes += 1;
        if self.check_timeout() {
            return [0; MAX_PLAYERS];
        }

        if self.rules.has_won(board, last_player) {
            let mut vec = [-WIN_SCORE + i32::from(ply); MAX_PLAYERS];
            vec[last_player as usize] = WIN_SCORE - i32::from(ply);
            return vec;
        }

        if board.is_full() {
            return [0; MAX_PLAYERS];
        }

        if depth == 0 {
            return evaluate_vector(board, self.rules, turn, self.player_count);
        }

        let moves = ordered_moves(board, None);
        let next_turn = (turn + 1) % self.player_count;
        let mut best_vec = [i32::MIN + 1; MAX_PLAYERS];

        for mv in moves {
            board.place_unchecked(turn, mv);
            let child_vec = self.maxn(board, depth - 1, ply + 1, next_turn, turn);
            board.undo(turn, mv);

            if self.timed_out {
                return [0; MAX_PLAYERS];
            }

            if child_vec[turn as usize] > best_vec[turn as usize] {
                best_vec = child_vec;
            }
        }

        best_vec
    }
}

/// True multi-player MaxN search: each player maximizes their own component
/// of the utility vector `[s_0, s_1, ..., s_{n-1}]`.
pub fn search_maxn(
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
            telemetry: SearchTelemetry::instant(AiAlgorithm::MaxN, 1, 1, start.elapsed()),
        });
    }

    if let Some((mv, score)) = find_tactical_move(board, rules, ai_player, player_count) {
        return Ok(SearchResult {
            best_move: mv,
            score,
            telemetry: SearchTelemetry::instant(AiAlgorithm::MaxN, 1, 1, start.elapsed()),
        });
    }

    let budget = Duration::from_millis(config.time_budget_ms.max(5));
    let mut ctx = MaxNContext {
        rules,
        player_count,
        deadline: start + budget,
        nodes: 0,
        timed_out: false,
    };

    let mut work_board = board.clone();
    let next_turn = (ai_player + 1) % player_count;
    let max_depth = config.max_depth.clamp(1, board.empty_count().min(12) as u8);

    let moves = ordered_moves(&work_board, None);
    let mut best_move = moves[0];
    let mut best_score = 0;
    let mut depth_reached = 0u8;

    for depth in 1..=max_depth {
        let mut iter_best_move = moves[0];
        let mut iter_best_score = i32::MIN + 1;

        for &mv in &moves {
            work_board.place_unchecked(ai_player, mv);
            let vec = ctx.maxn(&mut work_board, depth - 1, 1, next_turn, ai_player);
            work_board.undo(ai_player, mv);

            if ctx.timed_out && depth > 1 {
                break;
            }

            let score = vec[ai_player as usize];
            if score > iter_best_score {
                iter_best_score = score;
                iter_best_move = mv;
            }
        }

        if ctx.timed_out && depth > 1 {
            break;
        }

        best_move = iter_best_move;
        best_score = iter_best_score;
        depth_reached = depth;

        if best_score >= WIN_SCORE - 100 {
            break;
        }
    }

    Ok(SearchResult {
        best_move,
        score: best_score,
        telemetry: SearchTelemetry::new(
            AiAlgorithm::MaxN,
            depth_reached.max(1),
            ctx.nodes,
            0,
            start.elapsed(),
        ),
    })
}
