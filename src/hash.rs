use rand::{self, rngs::StdRng, RngCore, SeedableRng};
use std::hash::Hash;

use crate::piece::Color;
use crate::{piece::Piece, Board};

pub(crate) const MAX_PIECE_INDEX: usize = 767;
const SEED: [u8; 32] = [
    148, 94, 120, 126, 227, 253, 25, 236, 41, 96, 70, 10, 53, 197, 51, 231, 204, 44, 136, 210, 102,
    129, 128, 230, 251, 207, 200, 134, 166, 125, 236, 147,
];

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Board {
    pub(crate) fn initialize_hash(&mut self) {
        let mut rng = StdRng::from_seed(SEED);
        for i in 0..self.hash_keys.len() {
            self.hash_keys[i] = rng.next_u64();
        }

        // Creating hash
        let mut h = 0u64;
        for (i, p) in self.into_iter().enumerate() {
            if p != Piece::Empty {
                h ^= self.hash_keys[hash_index(p, i)]
            }
        }
        // p on square 63 would be 767
        let mut next_index = MAX_PIECE_INDEX + 1;
        if self.color_to_move() == Color::Black {
            h ^= self.hash_keys[next_index];
        }
        next_index += 1;
        for c in self.castle {
            if c {
                h ^= self.hash_keys[next_index];
            }
            next_index += 1;
        }
        if let Some(pos) = self.ep_target {
            h ^= self.hash_keys[next_index + pos.file() as usize];
        }
        self.hash = h;
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }
}

pub(crate) fn hash_index(p: Piece, index: usize) -> usize {
    (p.index() << 6) + index
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{moves::Move, Board};

    #[test]
    fn test_hash() {
        let mut board = Board::new();
        let initial = board.hash;
        board.make(Move::from_str("a2a3").unwrap()).unwrap();
        let after = board.hash;
        assert_ne!(initial, board.hash);
        board.unmake();
        assert_eq!(initial, board.hash);
        board.make(Move::from_str("a2a3").unwrap()).unwrap();
        assert_eq!(after, board.hash);
    }

    #[test]
    fn test_circular() {
        let mut board = Board::new();
        let initial = board.hash;
        let mvs = ["g1h3", "b8c6", "h3g1", "c6b8"];
        for m in mvs {
            board.make(m.parse().unwrap()).unwrap();
        }
        assert_eq!(initial, board.hash);
    }
}
