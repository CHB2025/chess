use std::fmt;

use moves::MoveState;
use piece::Piece;
use position::Position;
use square::Square;

use self::position::default_position;

pub mod error;
pub mod fen;
pub mod hash;
pub mod moves;
pub mod piece;
pub(crate) mod position;
pub mod square;

#[derive(Clone)]
pub struct Board {
    position: Position, // K,Q,B,N,R,P,-,-,k,q,b,n,r,p. So i & 8 == 0 = is White, i & 7 = Piece
    white_to_move: bool,
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
        let board = self.to_board_representation();
        let mut output = String::new();
        for row in 0..8 {
            output.push('|');
            for p in board[(row << 3)..((row + 1) << 3)].iter().rev() {
                output.push_str(&format!(" {p} |"));
            }
            output.push('\n')
        }
        write!(f, "{output}")
    }
}

impl IntoIterator for &Board {
    type Item = Piece;
    type IntoIter = std::array::IntoIter<Self::Item, 64>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_board_representation().into_iter()
    }
}

impl std::default::Default for Board {
    fn default() -> Self {
        let mut response = Self {
            position: default_position(),
            white_to_move: true,
            castle: [true; 4],
            ep_target: None,
            halfmove: 0,
            fullmove: 1, // This maybe should be 1?
            move_history: Vec::new(),
            hash: 0,
            hash_keys: [0; 781],
        };
        response.initialize_hash();
        response
    }
}

impl Board {
    fn to_board_representation(&self) -> [Piece; 64] {
        let mut board: [Piece; 64] = [Piece::Empty; 64];

        for index in 0..self.position.len() {
            let mut v = self.position[index];
            while v.leading_zeros() != u64::BITS {
                let first_bit = 63 - v.leading_zeros();
                v &= !(1 << first_bit);
                board[first_bit as usize] = (index as u8).try_into().unwrap();
            }
        }
        board
    }

    pub fn piece_at(&self, index: Square) -> Piece {
        let mask = 1u64 << index.index();
        for (i, p) in self.position.iter().enumerate() {
            if p & mask != 0 {
                return (i as u8).try_into().unwrap();
            }
        }
        unreachable!();
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_white_to_move(&self) -> bool {
        self.white_to_move
    }
}

#[cfg(test)]
mod tests {
    use crate::Board;

    #[test]
    fn test_default() {
        let game = Board::default();
        assert_eq!(game.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }
}
