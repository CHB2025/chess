use core::fmt;
use std::{default, ops};

use crate::piece::{Color, Piece, PieceType};
use crate::square::Square;

pub type Bitboard = u64;

/// Chess board representation using both bitboards and piecewise representation
#[derive(Clone, Copy)]
pub struct Position {
    bitboards: [Bitboard; 13],
    pieces: [Piece; 64],
}

impl default::Default for Position {
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            bitboards: [
                0x08 << 56,
                0x08,
                0x10 << 56,
                0x10,
                0x24 << 56,
                0x24,
                0x42 << 56,
                0x42,
                0x81 << 56,
                0x81,
                0xff << 48,
                0xff << 8,
                0xffffffff << 16,
            ],
            pieces: [
                Piece::Filled(PieceType::Rook, Color::Black),
                Piece::Filled(PieceType::Knight, Color::Black),
                Piece::Filled(PieceType::Bishop, Color::Black),
                Piece::Filled(PieceType::King, Color::Black),
                Piece::Filled(PieceType::Queen, Color::Black),
                Piece::Filled(PieceType::Bishop, Color::Black),
                Piece::Filled(PieceType::Knight, Color::Black),
                Piece::Filled(PieceType::Rook, Color::Black),
                // White Pawns
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                Piece::Filled(PieceType::Pawn, Color::Black),
                // Blank rows
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                //Black Pawns
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                Piece::Filled(PieceType::Pawn, Color::White),
                // Other black pieces
                Piece::Filled(PieceType::Rook, Color::White),
                Piece::Filled(PieceType::Knight, Color::White),
                Piece::Filled(PieceType::Bishop, Color::White),
                Piece::Filled(PieceType::King, Color::White),
                Piece::Filled(PieceType::Queen, Color::White),
                Piece::Filled(PieceType::Bishop, Color::White),
                Piece::Filled(PieceType::Knight, Color::White),
                Piece::Filled(PieceType::Rook, Color::White),
            ],
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for rank in 0..8 {
            let mut empty_squares = 0;
            for p in self.pieces[(rank << 3)..((rank + 1) << 3)].iter().rev() {
                if p == &Piece::Empty {
                    empty_squares += 1;
                    continue;
                }
                if empty_squares != 0 {
                    output += empty_squares.to_string().as_str();
                    empty_squares = 0;
                }
                output += p.to_string().as_str()
            }
            if empty_squares != 0 {
                output += &format!("{}", empty_squares);
            }
            if rank != 7 {
                output.push('/');
            }
        }
        write!(f, "{}", output)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for row in 0..8 {
            output.push_str(&format!("{}|", 8 - row));
            for p in self.pieces[(row << 3)..((row + 1) << 3)].iter().rev() {
                output.push_str(&format!(" {p} |"));
            }
            output.push('\n')
        }
        for col in 0..8 {
            output.push_str(&format!("   {}", char::from(b'a' + col)));
        }
        write!(f, "{output}")
    }
}

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

impl ops::Index<Square> for [Piece; 64] {
    type Output = Piece;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index.index() as usize]
    }
}
impl ops::IndexMut<Square> for [Piece; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index.index() as usize]
    }
}

impl Position {
    /// Creates a new, completely empty board representation
    pub fn empty() -> Self {
        Self {
            bitboards: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffffffffffffffff],
            pieces: [Piece::Empty; 64],
        }
    }

    /// Puts the provided piece in the provided square. Returns the piece that was replaced
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.pieces[square];
        self.pieces[square] = piece;
        let map = 1 << square.index();
        self.bitboards[replaced] &= !map;
        self.bitboards[piece] |= map;
        replaced
    }

    /// Clears the provided square. Returns the piece that previously held that position
    pub fn clear(&mut self, square: Square) -> Piece {
        self.put(Piece::Empty, square)
    }

    /// Moves the piece at `from` to `to`. Returns the piece that was replaced at `to`.
    pub fn r#move(&mut self, from: Square, to: Square) -> Piece {
        let piece = self.clear(from);
        self.put(piece, to)
    }

    pub fn move_replace(&mut self, from: Square, to: Square, replacement: Piece) -> Piece {
        let piece = self.put(replacement, from);
        self.put(piece, to)
    }

    pub fn team_pieces(&self, color: Color) -> Bitboard {
        // Pieces are every-other
        let range = Piece::Filled(PieceType::King, color).index()
            ..=Piece::Filled(PieceType::Pawn, color).index();
        let mut team = 0;
        for i in range.step_by(2) {
            team |= self.bitboards[i];
        }
        team
    }
}
