use crate::dir::Dir;
use crate::piece::{Color, Piece, PieceType};
use crate::square::Square;

use super::{Bitboard, Position};


impl Position {
    // Want to check for attackers. Return both checks and pins.
    // Allows move generation to make sure all undefended attacks are blocked and
    // defenders stay blocking attacks
    // Doesn't help for castling or moving into check...
    pub fn calc_pins_by_square(&self, square: Square) -> Bitboard {
        let piece = self[square];
        match piece {
            Piece::Filled(_, color) => self.calc_pins(square, color),
            Piece::Empty => !0,
        }
    }

    fn calc_pins(&self, square: Square, color: Color) -> Bitboard {
        let king_bitboard = self[Piece::Filled(PieceType::King, color)];
        if king_bitboard == 0 {
            return !0u64; // Needs to be able to work before the king is on the board (when
                          // building the board with fens)
        }
        let king = Square(63u8 - self[Piece::Filled(PieceType::King, color)].leading_zeros() as u8);
        if king == square {
            return !0u64;
        }
        // 0-1 will match 
        let dir = if king.rank() == square.rank() {
            if king > square { 
                Dir::East
            } else {
                Dir::West
            }
        } else if king.file() == square.file() {
            if  king > square {
                Dir::North
            } else {
                Dir::South
            }
        } else if king.diagonal() == square.diagonal() {
            // Not sure if this is accurate
            if king > square {
                Dir::SouWest
            } else {
                Dir::NorEast
            }
        } else if king.anti_diagonal() == square.anti_diagonal() {
            //Not sure if this is accurate
            if square > king {
                Dir::SouEast 
            } else {
                Dir::NorWest
            }
        } else {
            return !0u64;
        };

        let free = self[Piece::Empty];
        let attacker = Piece::Filled(dir.piece_type(), !color);
        let cap = self[attacker] | self[Piece::Filled(PieceType::Queen, !color)];
        pins(king, free, cap, square, dir)
    }

}

fn pins(
    initial: Square,
    free: Bitboard,
    cap: Bitboard,
    def: Square,
    dir: Dir,
) -> Bitboard {
    let mut output = 0u64;
    let mut pass = 1;
    let mut calc_end = |x: u64| {
        if x & (1 << def.index()) > 0 {
            pass -= 1;
        }
        x & (free | (1 << def.index()))
    };
    let mut mv = dir.shift(1 << initial.index());
    let mut end = calc_end(mv);
    let mut attacks = mv & cap;
    let mut hit = attacks != 0;

    while end > 0 || attacks > 0 {
        output |= end;
        output |= attacks;
        mv = dir.shift(end);
        end = calc_end(mv);
        attacks = mv & cap;
        if attacks != 0 {
            hit = true;
        }
    }
    if hit && pass == 0 {
        output
    } else {
        !0
    }
}
