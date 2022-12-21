use core::fmt;
use std::{default, ops};

use crate::piece::{Color, Piece, PieceType};
use crate::ray::Ray;
use crate::square::Square;

pub mod attacks;

pub type Bitboard = u64;

/// Chess board representation using both bitboards and piecewise representation
#[derive(Clone)]
pub struct Position {
    bitboards: [Bitboard; 13],
    attacks: [Bitboard; 2],
    pins: [Bitboard; 2],
    checks: [Bitboard; 2],
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
            attacks: [0xff << 40, 0xff << 16],
            pins: [0, 0],
            checks: [!0, !0],
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

impl Position {
    /// Creates a new, completely empty board representation
    pub fn empty() -> Self {
        Self {
            bitboards: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffffffffffffffff],
            attacks: [0, 0],
            pins: [0, 0],
            checks: [!0, !0],
            pieces: [Piece::Empty; 64],
        }
    }

    fn update_attacks_and_pins(&mut self) {
        self.update_pins_and_checks(Color::White);
        self.update_pins_and_checks(Color::Black);
        self.attacks[0] = self.gen_attacks(Color::White);
        self.attacks[1] = self.gen_attacks(Color::Black);
    }

    // Doesn't update attacks or pins
    fn internal_put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.pieces[square];
        self.pieces[square] = piece;
        let map = 1 << square.index();
        self.bitboards[replaced] &= !map;
        self.bitboards[piece] |= map;

        replaced
    }

    /// Puts the provided piece in the provided square. Returns the piece that was replaced
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.internal_put(piece, square);
        self.update_attacks_and_pins();

        replaced
    }

    /// Clears the provided square. Returns the piece that previously held that position
    pub fn clear(&mut self, square: Square) -> Piece {
        let removed = self.internal_put(Piece::Empty, square);
        self.update_attacks_and_pins();
        removed
    }

    /// Moves the piece at `from` to `to`. Returns the piece that was replaced at `to`.
    pub fn r#move(&mut self, from: Square, to: Square) -> Piece {
        self.move_replace(from, to, Piece::Empty)
    }

    pub fn move_replace(&mut self, from: Square, to: Square, replacement: Piece) -> Piece {
        let piece = self.internal_put(replacement, from);
        let capture = self.internal_put(piece, to);
        self.update_attacks_and_pins();
        capture
    }

    /// Returns the square occupied by the king of the provided color.
    /// Panics if the king is not on the board
    pub fn king(&self, color: Color) -> Square {
        let bitboard = self[Piece::Filled(PieceType::King, color)];
        if bitboard == 0 {
            panic!("King is not on the board")
        }
        Square(63 - bitboard.leading_zeros() as u8)
    }

    pub fn king_exists(&self, color: Color) -> bool {
        self[Piece::Filled(PieceType::King, color)] != 0
    }

    pub fn pieces_by_color(&self, color: Color) -> Bitboard {
        let range = Piece::Filled(PieceType::King, color).index()
            ..=Piece::Filled(PieceType::Pawn, color).index();
        range.step_by(2).fold(0, |team, i| team | self.bitboards[i])
    }

    pub fn attacks_by_color(&self, color: Color) -> Bitboard {
        self.attacks[color as usize]
    }

    pub fn pin_on_square(&self, square: Square) -> Option<Ray> {
        let piece = self[square];
        match piece {
            Piece::Filled(_, color) => {
                if self.pins[color as usize] & square.mask() == square.mask() {
                    Ray::from(self.king(color), square)
                } else {
                    None
                }
            }
            Piece::Empty => None,
        }
    }
    pub fn pins_on_color(&self, color: Color) -> Bitboard {
        self.pins[color as usize]
    }
    pub fn check_restrictions(&self, color: Color) -> Bitboard {
        self.checks[color as usize]
    }

    pub fn in_check(&self, color: Color) -> bool {
        self.checks[color as usize] != !0
    }
}
