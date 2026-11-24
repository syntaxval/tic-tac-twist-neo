use crate::error::EngineError;

pub const MIN_BOARD_SIZE: u8 = 3;
pub const MAX_BOARD_SIZE: u8 = 10;
pub const MAX_PLAYERS: usize = 4;

/// Compact, zero-allocation board backed by `u128` bitboards (one per player).
///
/// A 10×10 board has 100 cells, which fits comfortably in a single `u128`
/// (bits `0..100`). This makes move make/unmake, occupancy checks, and
/// win-mask evaluation single-cycle bitwise operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: u8,
    cell_count: u8,
    bitboards: [u128; MAX_PLAYERS],
    occupied: u128,
    full_mask: u128,
}

impl Board {
    pub fn new(size: u8) -> Result<Self, EngineError> {
        if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
            return Err(EngineError::InvalidBoardSize {
                size,
                min: MIN_BOARD_SIZE,
                max: MAX_BOARD_SIZE,
            });
        }

        let cell_count = size * size;
        let full_mask = (1u128 << cell_count) - 1;

        Ok(Self {
            size,
            cell_count,
            bitboards: [0; MAX_PLAYERS],
            occupied: 0,
            full_mask,
        })
    }

    #[inline]
    pub fn size(&self) -> u8 {
        self.size
    }

    #[inline]
    pub fn cell_count(&self) -> u8 {
        self.cell_count
    }

    #[inline]
    pub fn player_bits(&self, player: u8) -> u128 {
        self.bitboards[player as usize]
    }

    #[inline]
    pub fn occupied(&self) -> u128 {
        self.occupied
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.occupied == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.occupied == self.full_mask
    }

    #[inline]
    pub fn empty_count(&self) -> u32 {
        ((!self.occupied) & self.full_mask).count_ones()
    }

    /// O(1) conversion from 1-based `(row, col)` to 0-based flat cell index.
    #[inline]
    pub fn coord_to_index(&self, row: u8, col: u8) -> Result<u8, EngineError> {
        if row == 0 || col == 0 || row > self.size || col > self.size {
            return Err(EngineError::InvalidCoordinates {
                row,
                col,
                size: self.size,
            });
        }
        Ok((row - 1) * self.size + (col - 1))
    }

    /// O(1) conversion from 0-based flat cell index to 1-based `(row, col)`.
    #[inline]
    pub fn index_to_coord(&self, index: u8) -> Result<(u8, u8), EngineError> {
        if index >= self.cell_count {
            return Err(EngineError::OutOfBounds {
                index,
                cell_count: self.cell_count,
            });
        }
        Ok((index / self.size + 1, index % self.size + 1))
    }

    #[inline]
    pub fn place(&mut self, player: u8, index: u8) -> Result<(), EngineError> {
        if (player as usize) >= MAX_PLAYERS {
            return Err(EngineError::InvalidPlayerId {
                player,
                max: MAX_PLAYERS,
            });
        }
        if index >= self.cell_count {
            return Err(EngineError::OutOfBounds {
                index,
                cell_count: self.cell_count,
            });
        }
        let bit = 1u128 << index;
        if (self.occupied & bit) != 0 {
            return Err(EngineError::CellOccupied { index });
        }
        self.bitboards[player as usize] |= bit;
        self.occupied |= bit;
        Ok(())
    }

    /// Fast unchecked make-move for internal search hot loops.
    #[inline]
    pub(crate) fn place_unchecked(&mut self, player: u8, index: u8) {
        let bit = 1u128 << index;
        self.bitboards[player as usize] |= bit;
        self.occupied |= bit;
    }

    /// Fast O(1) unmake-move for search backtracking and undo.
    #[inline]
    pub fn undo(&mut self, player: u8, index: u8) {
        let bit = 1u128 << index;
        self.bitboards[player as usize] &= !bit;
        self.occupied &= !bit;
    }

    #[inline]
    pub fn cell_owner(&self, index: u8) -> Option<u8> {
        if index >= self.cell_count {
            return None;
        }
        let bit = 1u128 << index;
        if (self.occupied & bit) == 0 {
            return None;
        }
        for (p, &bb) in self.bitboards.iter().enumerate() {
            if (bb & bit) != 0 {
                return Some(p as u8);
            }
        }
        None
    }

    /// Zero-allocation iterator over all empty cell indices.
    #[inline]
    pub fn empty_cells(&self) -> EmptyCellsIter {
        EmptyCellsIter {
            bits: (!self.occupied) & self.full_mask,
        }
    }

    /// Flat snapshot of cell owners (`None` for empty) for UI/CLI rendering.
    pub fn cells_snapshot(&self) -> Vec<Option<u8>> {
        (0..self.cell_count).map(|i| self.cell_owner(i)).collect()
    }
}

pub struct EmptyCellsIter {
    bits: u128,
}

impl Iterator for EmptyCellsIter {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let idx = self.bits.trailing_zeros() as u8;
        self.bits &= self.bits - 1;
        Some(idx)
    }
}
