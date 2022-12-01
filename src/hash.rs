use rand::{self, RngCore, SeedableRng};
use std::hash::Hash;

use crate::{piece::Piece, Board};

pub(crate) const MAX_PIECE_INDEX: usize = 767;
const SEED: u64 = 0xd635879da32ff6c5;

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Board {
    pub(crate) fn initialize_hash(&mut self) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
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
        if !self.white_to_move {
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
            let col = (pos.index() & 0b111) as usize;
            h ^= self.hash_keys[next_index + col - 1]
        }
        self.hash = h;
    }


    pub fn get_hash(&self) -> u64 {
        return self.hash;
    }
}

pub(crate) fn hash_index(p: Piece, index: usize) -> usize {
    let p_index = p.index();
    // Because piece is only from 0-6 it needs to be first to minimize space needed
    //       Piece type               +1 if black               Location
    return ((((p_index & 0b111) << 1) + !p.is_white() as usize) << 6) + index;
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
