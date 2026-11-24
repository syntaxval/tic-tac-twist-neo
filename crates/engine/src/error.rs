use std::fmt;

/// Domain errors returned by the pure game engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidBoardSize { size: u8, min: u8, max: u8 },
    InvalidWinLength { win_length: u8, min: u8, max: u8 },
    InvalidPlayerCount { count: usize, min: usize, max: usize },
    InvalidPlayerId { player: u8, max: usize },
    OutOfBounds { index: u8, cell_count: u8 },
    InvalidCoordinates { row: u8, col: u8, size: u8 },
    CellOccupied { index: u8 },
    GameAlreadyOver,
    NotAnAiTurn,
    NoMovesToUndo,
    NoLegalMoves,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBoardSize { size, min, max } => {
                write!(f, "invalid board size {size}: must be between {min} and {max}")
            }
            Self::InvalidWinLength { win_length, min, max } => {
                write!(f, "invalid win length {win_length}: must be between {min} and {max}")
            }
            Self::InvalidPlayerCount { count, min, max } => {
                write!(f, "invalid player count {count}: must be between {min} and {max}")
            }
            Self::InvalidPlayerId { player, max } => {
                write!(f, "invalid player id {player}: must be < {max}")
            }
            Self::OutOfBounds { index, cell_count } => {
                write!(f, "cell index {index} is out of bounds (0..{cell_count})")
            }
            Self::InvalidCoordinates { row, col, size } => {
                write!(f, "coordinates ({row},{col}) are out of bounds for {size}x{size} board")
            }
            Self::CellOccupied { index } => write!(f, "cell {index} is already occupied"),
            Self::GameAlreadyOver => write!(f, "game is already over"),
            Self::NotAnAiTurn => write!(f, "current player is not an AI"),
            Self::NoMovesToUndo => write!(f, "no moves to undo"),
            Self::NoLegalMoves => write!(f, "no legal moves available"),
        }
    }
}

impl std::error::Error for EngineError {}
