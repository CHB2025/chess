use rand::{self, rngs::StdRng, RngCore, SeedableRng};

use crate::piece::Color;
use crate::{piece::Piece, Board};
use crate::{Castle, Square};

pub(crate) const MAX_PIECE_INDEX: usize = 767;
const SEED: [u8; 32] = [
    148, 94, 120, 126, 227, 253, 25, 236, 41, 96, 70, 10, 53, 197, 51, 231, 204, 44, 136, 210, 102,
    129, 128, 230, 251, 207, 200, 134, 166, 125, 236, 147,
];

// May make the zobrist key generation public so other board representations can share same keys?
#[inline]
pub(super) fn zobrist_keys() -> [u64; 781] {
    seeded_zobrist_keys(SEED)
}
#[inline]
pub(super) fn seeded_zobrist_keys(seed: [u8; 32]) -> [u64; 781] {
    let mut keys = [0u64; 781];
    let mut rng = StdRng::from_seed(seed);
    keys.iter_mut().for_each(|key| {
        *key = rng.next_u64();
    });
    keys
}

pub(super) fn toggle_color_hash(board: &mut Board) {
    board.hash ^= board.hash_keys[MAX_PIECE_INDEX + 1];
}

pub(super) fn update_castle_hash(board: &mut Board, color: Color, old: Castle, new: Castle) {
    let index = match color {
        Color::White => MAX_PIECE_INDEX + 2,
        Color::Black => MAX_PIECE_INDEX + 4,
    };
    if old.get_king_side() != new.get_king_side() {
        board.hash ^= board.hash_keys[index];
    }
    if old.get_queen_side() != new.get_queen_side() {
        board.hash ^= board.hash_keys[index + 1];
    }
}

pub(super) fn toggle_ep_hash(board: &mut Board, square: Square) {
    board.hash ^= board.hash_keys[MAX_PIECE_INDEX + 6 + square.file() as usize];
}

pub(super) fn increment_hash(board: &mut Board, piece: Piece, square: Square) {
    board.hash ^= board.hash_keys[hash_index(piece, square.index().into())];
}

pub(super) fn hash_index(p: Piece, index: usize) -> usize {
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
