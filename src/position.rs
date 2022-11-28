use std::ops;

use crate::piece::Piece;

pub type Bitboard = u64;

pub type Position = [Bitboard; 14];

impl ops::Index<Piece> for Position {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        return &self[index.index()];
    }
}

impl ops::IndexMut<Piece> for Position {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        return &mut self[index.index()];
    }
}
