use crate::board::{Board, MAX_PLAYERS};

const TT_SIZE: usize = 1 << 16; // 65,536 entries
const TT_MASK: usize = TT_SIZE - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TtEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
    pub best_move: u8,
}

impl Default for TtEntry {
    fn default() -> Self {
        Self {
            key: 0,
            depth: 0,
            score: 0,
            bound: Bound::Exact,
            best_move: u8::MAX,
        }
    }
}

pub struct ZobristTable {
    cell_keys: [[u64; 100]; MAX_PLAYERS],
    turn_keys: [u64; MAX_PLAYERS],
}

impl Default for ZobristTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ZobristTable {
    pub fn new() -> Self {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut next_u64 = || {
            seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };

        let mut cell_keys = [[0u64; 100]; MAX_PLAYERS];
        for player_keys in &mut cell_keys {
            for key in player_keys.iter_mut() {
                *key = next_u64();
            }
        }

        let mut turn_keys = [0u64; MAX_PLAYERS];
        for key in &mut turn_keys {
            *key = next_u64();
        }

        Self {
            cell_keys,
            turn_keys,
        }
    }

    #[inline]
    pub fn cell_key(&self, player: u8, cell: u8) -> u64 {
        self.cell_keys[player as usize][cell as usize]
    }

    #[inline]
    pub fn turn_key(&self, turn: u8) -> u64 {
        self.turn_keys[turn as usize]
    }

    pub fn hash_board(&self, board: &Board, player_count: u8, turn: u8) -> u64 {
        let mut h = self.turn_key(turn);
        for p in 0..player_count {
            let mut bits = board.player_bits(p);
            while bits != 0 {
                let idx = bits.trailing_zeros() as u8;
                h ^= self.cell_key(p, idx);
                bits &= bits - 1;
            }
        }
        h
    }
}

pub struct TranspositionTable {
    entries: Vec<TtEntry>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: vec![TtEntry::default(); TT_SIZE],
        }
    }

    #[inline]
    pub fn get(&self, key: u64) -> Option<TtEntry> {
        let entry = self.entries[(key as usize) & TT_MASK];
        if entry.key == key && entry.best_move != u8::MAX {
            Some(entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn store(&mut self, key: u64, depth: u8, score: i32, bound: Bound, best_move: u8) {
        let slot = &mut self.entries[(key as usize) & TT_MASK];
        if slot.key != key || depth >= slot.depth {
            *slot = TtEntry {
                key,
                depth,
                score,
                bound,
                best_move,
            };
        }
    }
}
