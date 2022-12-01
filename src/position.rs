use std::ops;

use crate::piece::Piece;

pub type Bitboard = u64;

pub type Position = [Bitboard; 14]; // K,Q,B,N,R,P,E,-,k,q,b,n,r,p

impl ops::Index<Piece> for Position {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index.index()]
    }
}

impl ops::IndexMut<Piece> for Position {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index.index()]
    }
}

pub(crate) fn default_position() -> Position {
    [0x08 << 56, 0x10 << 56, 0x24 << 56, 0x42 << 56, 0x81 << 56, 0xff << 48, 0xffffffff << 16, 0, 0x08, 0x10, 0x24, 0x42, 0x81, 0xff << 8]
}
