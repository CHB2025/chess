use std::{default, fmt, ops};

use moves::MoveState;
use piece::Piece;
use square::Square;

use self::piece::Color;
use self::position::Position;

pub mod dir;
pub mod error;
pub mod fen;
pub mod hash;
pub mod moves;
pub mod piece;
pub(crate) mod position;
pub mod ray;
pub mod square;

#[derive(Clone)]
pub struct Board {
    position: Position,
    color_to_move: Color,
    castle: [bool; 4],
    ep_target: Option<Square>,
    halfmove: u32,
    fullmove: u32,
    move_history: Vec<MoveState>,
    hash: u64,
    hash_keys: [u64; 781],
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.position)
    }
}

impl IntoIterator for &Board {
    type Item = Piece;
    type IntoIter = std::array::IntoIter<Self::Item, 64>;

    /// Returns an iterator of all the pieces on the board in big-endian order
    /// (h8-a1).
    fn into_iter(self) -> Self::IntoIter {
        self.position.into_iter()
    }
}

impl default::Default for Board {
    fn default() -> Self {
        let mut response = Self {
            position: Position::default(),
            color_to_move: Color::White,
            castle: [true; 4],
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

impl ops::Index<Square> for Board {
    type Output = Piece;

    fn index(&self, index: Square) -> &Self::Output {
        &self.position[index]
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_white_to_move(&self) -> bool {
        self.color_to_move == Color::White
    }
    pub fn color_to_move(&self) -> Color {
        self.color_to_move
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
    }
}
