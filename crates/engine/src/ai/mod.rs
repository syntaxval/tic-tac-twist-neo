pub mod heuristic;
pub mod maxn;
pub mod paranoid;
pub mod zobrist;

use serde::{Deserialize, Serialize};
use web_time::{Duration, Instant};

use crate::ai::heuristic::{center_cell, evaluate_perspective, find_tactical_move, ordered_moves};
use crate::ai::maxn::search_maxn;
use crate::ai::paranoid::search_paranoid;
use crate::board::Board;
use crate::error::EngineError;
use crate::rules::RuleSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiAlgorithm {
    Paranoid,
    MaxN,
    Greedy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub algorithm: AiAlgorithm,
    pub time_budget_ms: u64,
    pub max_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTelemetry {
    pub algorithm: AiAlgorithm,
    pub depth_reached: u8,
    pub nodes_evaluated: u64,
    pub tt_hits: u64,
    pub elapsed_ms: f64,
    pub nodes_per_sec: u64,
}

impl SearchTelemetry {
    pub fn new(
        algorithm: AiAlgorithm,
        depth_reached: u8,
        nodes_evaluated: u64,
        tt_hits: u64,
        elapsed: Duration,
    ) -> Self {
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let secs = elapsed.as_secs_f64().max(1e-6);
        let nodes_per_sec = (nodes_evaluated as f64 / secs) as u64;
        Self {
            algorithm,
            depth_reached,
            nodes_evaluated,
            tt_hits,
            elapsed_ms,
            nodes_per_sec,
        }
    }

    pub fn instant(
        algorithm: AiAlgorithm,
        depth_reached: u8,
        nodes_evaluated: u64,
        elapsed: Duration,
    ) -> Self {
        Self::new(algorithm, depth_reached, nodes_evaluated, 0, elapsed)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub best_move: u8,
    pub score: i32,
    pub telemetry: SearchTelemetry,
}

/// Dispatches AI move selection to the configured search strategy.
pub fn choose_move(
    board: &Board,
    rules: &RuleSet,
    ai_player: u8,
    player_count: u8,
    config: &AiConfig,
) -> Result<SearchResult, EngineError> {
    match config.algorithm {
        AiAlgorithm::Paranoid => search_paranoid(board, rules, ai_player, player_count, config),
        AiAlgorithm::MaxN => search_maxn(board, rules, ai_player, player_count, config),
        AiAlgorithm::Greedy => search_greedy(board, rules, ai_player, player_count),
    }
}

/// 1-ply heuristic baseline (fast reference point for visual benchmarks).
fn search_greedy(
    board: &Board,
    rules: &RuleSet,
    ai_player: u8,
    player_count: u8,
) -> Result<SearchResult, EngineError> {
    if board.is_full() {
        return Err(EngineError::NoLegalMoves);
    }

    let start = Instant::now();

    if board.is_empty() {
        return Ok(SearchResult {
            best_move: center_cell(board),
            score: 0,
            telemetry: SearchTelemetry::instant(AiAlgorithm::Greedy, 1, 1, start.elapsed()),
        });
    }

    if let Some((mv, score)) = find_tactical_move(board, rules, ai_player, player_count) {
        return Ok(SearchResult {
            best_move: mv,
            score,
            telemetry: SearchTelemetry::instant(AiAlgorithm::Greedy, 1, 1, start.elapsed()),
        });
    }

    let mut work_board = board.clone();
    let moves = ordered_moves(&work_board, None);
    let next_turn = (ai_player + 1) % player_count;

    let mut best_move = moves[0];
    let mut best_score = i32::MIN;
    let mut nodes = 0u64;

    for mv in moves {
        work_board.place_unchecked(ai_player, mv);
        let score = evaluate_perspective(&work_board, rules, ai_player, next_turn, player_count);
        work_board.undo(ai_player, mv);
        nodes += 1;

        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    Ok(SearchResult {
        best_move,
        score: best_score,
        telemetry: SearchTelemetry::new(AiAlgorithm::Greedy, 1, nodes, 0, start.elapsed()),
    })
}
