use core::fmt;
use std::default;
use std::str::FromStr;

use self::action::Action;
use crate::{Bitboard, BoardError, Color, ErrorKind, Piece, Ray, Square};

pub mod action;
pub mod attacks;
pub mod index;

/// Chess board representation using both bitboards and piecewise representation
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position {
    bitboards: [Bitboard; 13],
    colors: [Bitboard; 2],
    attacks: Bitboard,
    pins: Bitboard,
    checks: Bitboard,
    color_to_move: Color,
    pieces: [Piece; 64],
}

impl FromStr for Position {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut position = Self::empty();
        let (b, ctm) = s.split_once(' ').ok_or(BoardError::new(
            ErrorKind::InvalidInput,
            "Invalid Position Input. Didn't include both color and position",
        ))?;
        let mut row_count = 0;
        let mut pos_count = 0;
        for (y, row) in b.split('/').enumerate() {
            let mut offset: usize = 0;
            for (x, symbol) in row.chars().rev().enumerate() {
                if symbol.is_numeric() {
                    let o = symbol.to_string().parse::<usize>()?;
                    pos_count += o;
                    offset += o - 1;
                    continue;
                }
                let p: Piece = symbol.try_into()?;
                let square: Square = ((y << 3) + x + offset).try_into()?;
                let bb = Bitboard::from(square);
                position.pieces[square] = p;
                position.bitboards[Piece::Empty] ^= bb;
                position.bitboards[p] |= bb;
                if let Some(color) = p.color() {
                    position.colors[color] |= bb;
                }
                pos_count += 1;
            }
            row_count += 1;
        }

        position.color_to_move = ctm.parse()?;

        if row_count != 8 || pos_count != 64 {
            println!("{} rows, {} positions", row_count, pos_count);
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Invalid Position Input. Row or Position count did not match expected",
            ));
        }
        position.update_attacks_and_pins();

        Ok(position)
    }
}

impl default::Default for Position {
    #[rustfmt::skip]
    fn default() -> Self {
         Self {
            bitboards: [
                Bitboard(0x08 << 56),
                Bitboard(0x08),
                Bitboard(0x10 << 56),
                Bitboard(0x10),
                Bitboard(0x24 << 56),
                Bitboard(0x24),
                Bitboard(0x42 << 56),
                Bitboard(0x42),
                Bitboard(0x81 << 56),
                Bitboard(0x81),
                Bitboard(0xff << 48),
                Bitboard(0xff << 8),
                Bitboard(0xffffffff << 16),
            ],
            colors: [ Bitboard(0xffff << 48), Bitboard(0xffff) ],
            attacks: Bitboard(0xffff7e),
            pins: Bitboard(0),
            checks: Bitboard(!0),
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
        write!(f, "{} {}", output, self.color_to_move)
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
            bitboards: [
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0),
                Bitboard(0xffffffffffffffff),
            ],
            colors: [Bitboard(0); 2],
            attacks: Bitboard(0),
            pins: Bitboard(0),
            checks: Bitboard(!0),
            color_to_move: Color::White,
            pieces: [Piece::Empty; 64],
        }
    }

    fn update_attacks_and_pins(&mut self) {
        self.update_pins_and_checks();
        self.attacks = self.gen_attacks(!self.color_to_move);
    }

    pub fn action<'a, T>(&'a mut self, arg: impl FnOnce(&mut Action<'a>) -> T) -> T {
        let mut action = Action { position: self };
        let response = arg(&mut action);
        action.complete();
        response
    }

    /// Returns the square occupied by the king of the provided color.
    /// Panics if the king is not on the board
    pub fn king(&self, color: Color) -> Square {
        self[Piece::king(color)]
            .first_square()
            .expect("King is not on the board")
    }

    pub fn king_exists(&self, color: Color) -> bool {
        !self[Piece::king(color)].is_empty()
    }

    pub fn attacks(&self) -> Bitboard {
        self.attacks
    }

    pub fn pin_on_square(&self, square: Square) -> Option<Ray> {
        let piece = self[square];
        match piece {
            Piece::Filled(_, color) => {
                if self.pins & Bitboard::from(square) == Bitboard::from(square) {
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
    pub fn pins(&self) -> Bitboard {
        self.pins
    }
    pub fn check_restrictions(&self) -> Bitboard {
        self.checks
    }

    pub fn in_check(&self) -> bool {
        self.checks != Bitboard(!0)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_from_string() {
        let fs = Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w").unwrap();
        assert_eq!(fs, Position::default());
    }
}
