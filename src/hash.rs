use rand::{self, RngCore, SeedableRng};
use std::hash::Hash;

use crate::{moves::MoveState, piece::Piece, Board};

const MAX_PIECE_INDEX: usize = 767;
const SEED: u64 = 0xd635879da32ff6c5;

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Board {
    pub fn initialize_hash(&mut self) {
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

    pub fn increment_hash(&mut self, ms: MoveState, p: Piece) {
        self.hash ^= self.hash_keys[hash_index(p, ms.mv.origin.index().into())];
        if ms.mv.promotion == Piece::Empty {
            self.hash ^= self.hash_keys[hash_index(p, ms.mv.dest.index().into())];
        } else {
            self.hash ^= self.hash_keys[hash_index(ms.mv.promotion, ms.mv.dest.index().into())];
        }

        let is_ep = if let Some(pos) = ms.ep_target {
            pos == ms.mv.dest && p == Piece::Pawn(p.is_white())
        } else {
            false
        };

        if ms.capture != Piece::Empty && !is_ep {
            self.hash ^= self.hash_keys[hash_index(ms.capture, ms.mv.dest.index().into())];
        }
        if is_ep {
            let index = (ms.mv.origin.index() & !0b111) | (ms.mv.dest.index() & 0b111);
            self.hash ^= self.hash_keys[hash_index(ms.capture, index.into())];
        }

        let is_castle = Piece::King(p.is_white()) == p
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() > ms.mv.origin.index();
        if is_castle {
            let rook = Piece::Rook(p.is_white());
            let r_origin = if is_ks_castle {
                ms.mv.origin.index() | 0b111
            } else {
                ms.mv.origin.index() & !0b111
            };
            let r_dest = if is_ks_castle {
                r_origin - 2
            } else {
                r_origin + 3
            };
            self.hash ^= self.hash_keys[hash_index(rook, r_origin.into())];
            self.hash ^= self.hash_keys[hash_index(rook, r_dest.into())];
        }

        let mut next_index = MAX_PIECE_INDEX + 1;
        self.hash ^= self.hash_keys[next_index];
        next_index += 1;

        for (i, c) in self.castle.iter().enumerate() {
            if *c != ms.castle[i] {
                self.hash ^= self.hash_keys[next_index];
            }
            next_index += 1;
        }
        if let Some(pos) = ms.ep_target {
            let col = (pos.index() & 0b111) as usize;
            self.hash ^= self.hash_keys[next_index + col - 1];
        }
        if let Some(pos) = self.ep_target {
            let col = (pos.index() & 0b111) as usize;
            self.hash ^= self.hash_keys[next_index + col - 1]
        }
    }
    pub fn get_hash(&self) -> u64 {
        return self.hash;
    }
}

fn hash_index(p: Piece, index: usize) -> usize {
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