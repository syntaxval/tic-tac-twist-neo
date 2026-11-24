use serde::{Deserialize, Serialize};

use crate::ai::{AiAlgorithm, AiConfig, SearchResult, choose_move};
use crate::board::{Board, MAX_PLAYERS};
use crate::error::EngineError;
use crate::rules::{RuleSet, WinKind};

pub const MIN_PLAYERS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PlayerKind {
    Human,
    Ai(AiConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub name: String,
    pub symbol: String,
    pub color: String,
    pub kind: PlayerKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub board_size: u8,
    pub win_length: u8,
    pub four_corners: bool,
    pub players: Vec<PlayerConfig>,
}

impl GameConfig {
    /// Classic 3-player setup matching the 2019 game, upgraded with Paranoid Alpha-Beta.
    pub fn default_classic(board_size: u8) -> Self {
        Self {
            board_size,
            win_length: board_size,
            four_corners: false,
            players: vec![
                PlayerConfig {
                    name: "Zalgo".into(),
                    symbol: "❤".into(),
                    color: "#e879f9".into(),
                    kind: PlayerKind::Ai(AiConfig {
                        algorithm: AiAlgorithm::Paranoid,
                        time_budget_ms: 120,
                        max_depth: 12,
                    }),
                },
                PlayerConfig {
                    name: "Brent".into(),
                    symbol: "✹".into(),
                    color: "#38bdf8".into(),
                    kind: PlayerKind::Human,
                },
                PlayerConfig {
                    name: "Josh".into(),
                    symbol: "✤".into(),
                    color: "#4ade80".into(),
                    kind: PlayerKind::Human,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Outcome {
    Ongoing,
    Win {
        player: u8,
        kind: WinKind,
        cells: Vec<u8>,
    },
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRecord {
    pub player: u8,
    pub cell: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSnapshot {
    pub board_size: u8,
    pub win_length: u8,
    pub four_corners: bool,
    pub cells: Vec<Option<u8>>,
    pub players: Vec<PlayerConfig>,
    pub current_turn: u8,
    pub outcome: Outcome,
    pub history: Vec<MoveRecord>,
}

/// Pure, I/O-free game state machine.
#[derive(Debug, Clone)]
pub struct Game {
    config: GameConfig,
    board: Board,
    rules: RuleSet,
    current_turn: u8,
    outcome: Outcome,
    history: Vec<MoveRecord>,
}

impl Game {
    pub fn new(config: GameConfig) -> Result<Self, EngineError> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&config.players.len()) {
            return Err(EngineError::InvalidPlayerCount {
                count: config.players.len(),
                min: MIN_PLAYERS,
                max: MAX_PLAYERS,
            });
        }

        let board = Board::new(config.board_size)?;
        let rules = RuleSet::build(config.board_size, config.win_length, config.four_corners)?;

        Ok(Self {
            config,
            board,
            rules,
            current_turn: 0,
            outcome: Outcome::Ongoing,
            history: Vec::new(),
        })
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn current_turn(&self) -> u8 {
        self.current_turn
    }

    pub fn current_player(&self) -> &PlayerConfig {
        &self.config.players[self.current_turn as usize]
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    pub fn history(&self) -> &[MoveRecord] {
        &self.history
    }

    pub fn is_over(&self) -> bool {
        !matches!(self.outcome, Outcome::Ongoing)
    }

    /// Plays a move for the active player at flat `cell` index.
    pub fn play_move(&mut self, cell: u8) -> Result<&Outcome, EngineError> {
        if self.is_over() {
            return Err(EngineError::GameAlreadyOver);
        }

        let player = self.current_turn;
        self.board.place(player, cell)?;
        self.history.push(MoveRecord { player, cell });

        if let Some(win_mask) = self.rules.check_winner(&self.board, player) {
            self.outcome = Outcome::Win {
                player,
                kind: win_mask.kind,
                cells: win_mask.cells.clone(),
            };
        } else if self.board.is_full() {
            self.outcome = Outcome::Draw;
        } else {
            self.current_turn = ((player as usize + 1) % self.config.players.len()) as u8;
        }

        Ok(&self.outcome)
    }

    /// Computes the active AI player's move without mutating game state.
    pub fn compute_ai_move(&self) -> Result<SearchResult, EngineError> {
        if self.is_over() {
            return Err(EngineError::GameAlreadyOver);
        }

        let ai_cfg = match &self.current_player().kind {
            PlayerKind::Ai(cfg) => cfg,
            PlayerKind::Human => return Err(EngineError::NotAnAiTurn),
        };

        choose_move(
            &self.board,
            &self.rules,
            self.current_turn,
            self.config.players.len() as u8,
            ai_cfg,
        )
    }

    /// Computes and immediately applies the active AI player's move.
    pub fn play_ai_turn(&mut self) -> Result<SearchResult, EngineError> {
        let result = self.compute_ai_move()?;
        self.play_move(result.best_move)?;
        Ok(result)
    }

    /// Undoes the last move and restores turn + ongoing status.
    pub fn undo(&mut self) -> Result<MoveRecord, EngineError> {
        let record = self.history.pop().ok_or(EngineError::NoMovesToUndo)?;
        self.board.undo(record.player, record.cell);
        self.current_turn = record.player;
        self.outcome = Outcome::Ongoing;
        Ok(record)
    }

    /// Resets the board while preserving the current configuration.
    pub fn reset(&mut self) {
        self.board = Board::new(self.config.board_size).expect("validated board size");
        self.current_turn = 0;
        self.outcome = Outcome::Ongoing;
        self.history.clear();
    }

    pub fn snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            board_size: self.config.board_size,
            win_length: self.config.win_length,
            four_corners: self.config.four_corners,
            cells: self.board.cells_snapshot(),
            players: self.config.players.clone(),
            current_turn: self.current_turn,
            outcome: self.outcome.clone(),
            history: self.history.clone(),
        }
    }
}
