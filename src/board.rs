use std::{default, fmt, ops};

use crate::{Color, MoveState, Piece, Square};

use self::position::Position;

pub mod fen;
pub mod generate;
pub mod hash;
pub mod make;
pub mod perft;
pub(crate) mod position;

#[derive(Clone)]
pub struct Board {
    position: Position,
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
        self.position.color_to_move() == Color::White
    }
    pub fn color_to_move(&self) -> Color {
        self.position.color_to_move()
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
