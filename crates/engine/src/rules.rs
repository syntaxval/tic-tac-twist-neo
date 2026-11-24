use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::error::EngineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WinKind {
    Row,
    Column,
    Diagonal,
    FourCorners,
}

/// Precomputed bitmask representing a single winning pattern on a board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinMask {
    pub mask: u128,
    pub len: u8,
    pub kind: WinKind,
    pub cells: Vec<u8>,
}

/// Strategy trait for composable win conditions.
pub trait WinRule: Send + Sync {
    fn generate_masks(&self, board_size: u8) -> Vec<WinMask>;
}

/// Classic horizontal, vertical, and diagonal contiguous segments of `win_length`.
pub struct LinesRule {
    pub win_length: u8,
}

impl WinRule for LinesRule {
    fn generate_masks(&self, board_size: u8) -> Vec<WinMask> {
        let n = board_size as usize;
        let k = self.win_length as usize;
        let mut masks = Vec::new();

        // Rows
        for r in 0..n {
            for c in 0..=(n - k) {
                let mut mask = 0u128;
                let mut cells = Vec::with_capacity(k);
                for offset in 0..k {
                    let idx = (r * n + c + offset) as u8;
                    mask |= 1u128 << idx;
                    cells.push(idx);
                }
                masks.push(WinMask {
                    mask,
                    len: self.win_length,
                    kind: WinKind::Row,
                    cells,
                });
            }
        }

        // Columns
        for c in 0..n {
            for r in 0..=(n - k) {
                let mut mask = 0u128;
                let mut cells = Vec::with_capacity(k);
                for offset in 0..k {
                    let idx = ((r + offset) * n + c) as u8;
                    mask |= 1u128 << idx;
                    cells.push(idx);
                }
                masks.push(WinMask {
                    mask,
                    len: self.win_length,
                    kind: WinKind::Column,
                    cells,
                });
            }
        }

        // Main diagonals (top-left to bottom-right)
        for r in 0..=(n - k) {
            for c in 0..=(n - k) {
                let mut mask = 0u128;
                let mut cells = Vec::with_capacity(k);
                for offset in 0..k {
                    let idx = ((r + offset) * n + (c + offset)) as u8;
                    mask |= 1u128 << idx;
                    cells.push(idx);
                }
                masks.push(WinMask {
                    mask,
                    len: self.win_length,
                    kind: WinKind::Diagonal,
                    cells,
                });
            }
        }

        // Anti-diagonals (top-right to bottom-left)
        for r in 0..=(n - k) {
            for c in (k - 1)..n {
                let mut mask = 0u128;
                let mut cells = Vec::with_capacity(k);
                for offset in 0..k {
                    let idx = ((r + offset) * n + (c - offset)) as u8;
                    mask |= 1u128 << idx;
                    cells.push(idx);
                }
                masks.push(WinMask {
                    mask,
                    len: self.win_length,
                    kind: WinKind::Diagonal,
                    cells,
                });
            }
        }

        masks
    }
}

/// Extension rule: owning all four outer corners of the board wins.
/// Always uses the actual `board_size` rather than a global config constant.
pub struct FourCornersRule;

impl WinRule for FourCornersRule {
    fn generate_masks(&self, board_size: u8) -> Vec<WinMask> {
        let cell_count = board_size * board_size;
        let corners = vec![0, board_size - 1, cell_count - board_size, cell_count - 1];
        let mask = corners.iter().fold(0u128, |acc, &idx| acc | (1u128 << idx));
        vec![WinMask {
            mask,
            len: 4,
            kind: WinKind::FourCorners,
            cells: corners,
        }]
    }
}

/// Compiled rule set holding all active winning bitmasks for a game.
#[derive(Debug, Clone)]
pub struct RuleSet {
    masks: Vec<WinMask>,
    win_length: u8,
    four_corners: bool,
}

impl RuleSet {
    pub fn build(
        board_size: u8,
        win_length: u8,
        four_corners: bool,
    ) -> Result<Self, EngineError> {
        if win_length < 3 || win_length > board_size {
            return Err(EngineError::InvalidWinLength {
                win_length,
                min: 3,
                max: board_size,
            });
        }

        let mut rules: Vec<Box<dyn WinRule>> = vec![Box::new(LinesRule { win_length })];
        if four_corners {
            rules.push(Box::new(FourCornersRule));
        }

        let masks = rules
            .into_iter()
            .flat_map(|rule| rule.generate_masks(board_size))
            .collect();

        Ok(Self {
            masks,
            win_length,
            four_corners,
        })
    }

    #[inline]
    pub fn masks(&self) -> &[WinMask] {
        &self.masks
    }

    #[inline]
    pub fn win_length(&self) -> u8 {
        self.win_length
    }

    #[inline]
    pub fn four_corners(&self) -> bool {
        self.four_corners
    }

    /// Checks whether `player` satisfies any winning pattern in O(masks) bitwise ops.
    #[inline]
    pub fn check_winner(&self, board: &Board, player: u8) -> Option<&WinMask> {
        let bits = board.player_bits(player);
        self.masks.iter().find(|m| (bits & m.mask) == m.mask)
    }

    /// Fast boolean win check for search hot loops.
    #[inline]
    pub fn has_won(&self, board: &Board, player: u8) -> bool {
        let bits = board.player_bits(player);
        self.masks.iter().any(|m| (bits & m.mask) == m.mask)
    }
}
