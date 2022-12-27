use rand::{self, rngs::StdRng, RngCore, SeedableRng};
use std::hash::Hash;

use crate::moves::MoveState;
use crate::piece::{Color, PieceKind};
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

    pub(super) fn increment_hash(&mut self, ms: MoveState, p: Piece) {
        self.hash ^= self.hash_keys[hash_index(p, ms.mv.origin.index().into())];
        if ms.mv.promotion == Piece::Empty {
            self.hash ^= self.hash_keys[hash_index(p, ms.mv.dest.index().into())];
        } else {
            self.hash ^= self.hash_keys[hash_index(ms.mv.promotion, ms.mv.dest.index().into())];
        }

        let is_ep = if let Some(pos) = ms.ep_target {
            pos == ms.mv.dest && p.kind() == Some(PieceKind::Pawn)
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

        let is_castle = p.is_kind(PieceKind::King)
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
        if is_castle {
            let rook = Piece::rook(!self.color_to_move());
            let r_origin = if is_ks_castle {
                ms.mv.origin.index() & !0b111
            } else {
                ms.mv.origin.index() | 0b111
            };
            let r_dest = if is_ks_castle {
                r_origin + 2
            } else {
                r_origin - 3
            };
            self.hash ^= self.hash_keys[hash_index(rook, r_origin.into())];
            self.hash ^= self.hash_keys[hash_index(rook, r_dest.into())];
        }
        let mut next_index = MAX_PIECE_INDEX + 1;
        // Changing sides
        self.hash ^= self.hash_keys[next_index];
        next_index += 1;

        for i in 0..4 {
            if self.castle[i] != ms.castle[i] {
                self.hash ^= self.hash_keys[next_index];
            }
            next_index += 1;
        }
        if let Some(pos) = ms.ep_target {
            self.hash ^= self.hash_keys[next_index + pos.file() as usize];
        }
        if let Some(pos) = self.ep_target {
            self.hash ^= self.hash_keys[next_index + pos.file() as usize]
        }
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
