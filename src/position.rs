use core::fmt;
use std::default;

use crate::piece::{Color, Piece};
use crate::ray::Ray;
use crate::square::Square;

pub mod attacks;
pub mod index;

pub type Bitboard = u64;

/// Chess board representation using both bitboards and piecewise representation
#[derive(Clone)]
pub struct Position {
    bitboards: [Bitboard; 13],
    colors: [Bitboard; 2],
    attacks: Bitboard,
    pins: Bitboard,
    checks: Bitboard,
    color_to_move: Color,
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
            colors: [ 0xffff << 40, 0xffff ],
            attacks: 0xff << 40,
            pins: 0,
            checks: !0,
            color_to_move: Color::White,
            pieces: [
                Piece::rook(Color::Black),
                Piece::knight(Color::Black),
                Piece::bishop(Color::Black),
                Piece::king(Color::Black),
                Piece::queen(Color::Black),
                Piece::bishop(Color::Black),
                Piece::knight(Color::Black),
                Piece::rook(Color::Black),
                // White Pawns
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                Piece::pawn(Color::Black),
                // Blank rows
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                //Black Pawns
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                Piece::pawn(Color::White),
                // Other black pieces
                Piece::rook(Color::White),
                Piece::knight(Color::White),
                Piece::bishop(Color::White),
                Piece::king(Color::White),
                Piece::queen(Color::White),
                Piece::bishop(Color::White),
                Piece::knight(Color::White),
                Piece::rook(Color::White),
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
impl Position {
    /// Creates a new, completely empty board representation
    pub fn empty() -> Self {
        Self {
            bitboards: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffffffffffffffff],
            colors: [0, 0],
            attacks: 0,
            pins: 0,
            checks: !0,
            color_to_move: Color::White,
            pieces: [Piece::Empty; 64],
        }
    }

    pub fn update_attacks_and_pins(&mut self) {
        self.update_pins_and_checks();
        self.attacks = self.gen_attacks(!self.color_to_move);
    }

    // Doesn't update attacks or pins
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.pieces[square];
        self.pieces[square] = piece;
        let map = 1 << square.index();
        self.bitboards[replaced] &= !map;
        if let Some(color) = replaced.color() {
            self.colors[color] &= !map;
        }
        self.bitboards[piece] |= map;
        if let Some(color) = piece.color() {
            self.colors[color] |= map;
        }

        replaced
    }

    /// Clears the provided square. Returns the piece that previously held that position
    pub fn clear(&mut self, square: Square) -> Piece {
        self.put(Piece::Empty, square)
    }

    /// Moves the piece at `from` to `to`. Returns the piece that was replaced at `to`.
    pub fn r#move(&mut self, from: Square, to: Square) -> Piece {
        self.move_replace(from, to, Piece::Empty)
    }

    pub fn move_replace(&mut self, from: Square, to: Square, replacement: Piece) -> Piece {
        let piece = self.put(replacement, from);
        self.put(piece, to)
    }

    /// Returns the square occupied by the king of the provided color.
    /// Panics if the king is not on the board
    pub fn king(&self, color: Color) -> Square {
        let bitboard = self[Piece::king(color)];
        if bitboard == 0 {
            panic!("King is not on the board")
        }
        Square(63 - bitboard.leading_zeros() as u8)
    }

    pub fn king_exists(&self, color: Color) -> bool {
        self[Piece::king(color)] != 0
    }

    pub fn attacks(&self) -> Bitboard {
        self.attacks
    }

    pub fn pin_on_square(&self, square: Square) -> Option<Ray> {
        let piece = self[square];
        match piece {
            Piece::Filled(_, color) => {
                if self.pins & square.mask() == square.mask() {
                    Ray::from(self.king(color), square)
                } else {
                    None
                }
            }
            Piece::Empty => None,
        }
    }
    pub fn color_to_move(&self) -> Color {
        self.color_to_move
    }
    pub fn switch_color_to_move(&mut self) {
        self.color_to_move = !self.color_to_move;
        self.update_attacks_and_pins();
    }
    pub fn pins(&self) -> Bitboard {
        self.pins
    }
    pub fn check_restrictions(&self) -> Bitboard {
        self.checks
    }

    pub fn in_check(&self) -> bool {
        self.checks != !0
    }
}
