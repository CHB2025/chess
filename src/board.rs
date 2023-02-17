use std::{default, fmt};

use crate::{Bitboard, Color, Move, MoveState, Piece, Ray, Square};

use self::action::Action;

mod action;
mod attacks;
mod fen;
mod generate;
mod hash;
mod index;
mod make;
pub mod move_gen;
mod perft;

#[derive(Clone)]
pub struct Board {
    bitboards: [Bitboard; 13],
    color_bitboards: [Bitboard; 2],
    attacks: Bitboard,
    pins: Bitboard,
    checkers: Bitboard,
    color_to_move: Color,
    pieces: [Piece; 64],
    castle: [bool; 4],
    ep_target: Option<Square>,
    halfmove: u32,
    fullmove: u32,
    move_history: Vec<MoveState>,
    hash: u64,
    hash_keys: [u64; 781],
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for row in 0..8 {
            output += &format!("{}|", 8 - row);
            for p in self.pieces[(row << 3)..((row + 1) << 3)].iter().rev() {
                output += &format!(" {p} |");
            }
            output += "\n";
        }
        for col in 0..8 {
            output += &format!("   {}", char::from(b'a' + col));
        }
        write!(f, "{output}")
    }
}

impl IntoIterator for &Board {
    type Item = Piece;
    type IntoIter = std::array::IntoIter<Self::Item, 64>;

    /// Returns an iterator of all the pieces on the board in big-endian order
    /// (h8-a1).
    fn into_iter(self) -> Self::IntoIter {
        self.pieces.into_iter()
    }
}

impl default::Default for Board {
    fn default() -> Self {
        let mut response = Self {
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
            color_bitboards: [Bitboard(0xffff << 48), Bitboard(0xffff)],
            attacks: Bitboard(0xffff7e),
            pins: Bitboard(0),
            checkers: Bitboard(!0),
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
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
                Piece::Empty,
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
            castle: [true; 4],
            color_to_move: Color::White,
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
            move_history: Vec::new(),
            hash: 0,
            hash_keys: [0; 781],
        };
        response.initialize_hash();
        response
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates and returns an empy board. Useful for setting up new positions
    pub fn empty() -> Self {
        let mut e = Self {
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
                Bitboard(u64::MAX),
            ],
            color_bitboards: [Bitboard(0); 2],
            attacks: Bitboard(0),
            pins: Bitboard(0),
            checkers: Bitboard(!0),
            pieces: [Piece::Empty; 64],
            color_to_move: Color::White,
            castle: [false; 4],
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
            move_history: Vec::new(),
            hash: 0,
            hash_keys: [0; 781],
        };
        e.initialize_hash();
        e
    }

    pub fn is_white_to_move(&self) -> bool {
        self.color_to_move == Color::White
    }
    pub fn color_to_move(&self) -> Color {
        self.color_to_move
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        //self.moves()
        move_gen::legal(self)
    }

    fn king(&self, color: Color) -> Square {
        self[Piece::king(color)]
            .first_square()
            .expect("King is not on the board")
    }

    fn king_exists(&self, color: Color) -> bool {
        !self[Piece::king(color)].is_empty()
    }

    fn pin_on_square(&self, square: Square) -> Option<Ray> {
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

    fn action<'a, T>(&'a mut self, arg: impl FnOnce(&mut Action<'a>) -> T) -> T {
        let mut action = Action { board: self };
        let response = arg(&mut action);
        action.complete();
        response
    }
}

#[cfg(test)]
mod tests {
    use crate::Board;

    #[test]
    fn test_default() {
        let game = Board::default();
        assert_eq!(
            game.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        for mv in game.moves() {
            println!("{}", mv);
        }
        assert_eq!(game.moves().len(), 20)
    }
}
