use std::ops;

use crate::piece::{Color, Piece};
use crate::square::Square;

use super::{Bitboard, Position};

// Public indexing operations
impl ops::Index<Piece> for Position {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self.bitboards[index.index()]
    }
}

impl ops::Index<Square> for Position {
    type Output = Piece;

    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index.index() as usize]
    }
}

impl ops::Index<Color> for Position {
    type Output = Bitboard;

    fn index(&self, index: Color) -> &Self::Output {
        &self.colors[index as usize]
    }
}

impl IntoIterator for &Position {
    type Item = Piece;

    type IntoIter = std::array::IntoIter<Self::Item, 64>;

    fn into_iter(self) -> Self::IntoIter {
        self.pieces.into_iter()
    }
}

// Re-implementing indexing on the underlying arrays rather than the struct itself
// so that the mutable indexing is not available outside the mod
impl ops::Index<Piece> for [Bitboard; 13] {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index.index()]
    }
}
impl ops::IndexMut<Piece> for [Bitboard; 13] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index.index()]
    }
}

impl<T> ops::Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index.index() as usize]
    }
}
impl<T> ops::IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index.index() as usize]
    }
}

impl<T> ops::Index<Color> for [T; 2] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T> ops::IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
