use std::fmt;

use moves::MoveState;
use piece::Piece;
use position::Square;

pub mod error;
pub mod fen;
pub mod hash;
pub mod moves;
pub mod piece;
pub mod position;

pub type Bitboard = u64;

#[derive(Clone)]
pub struct Board {
    pieces: [Bitboard; 14], // K,Q,B,N,R,P,-,-,k,q,b,n,r,p. So i & 8 == 0 = is White, i & 7 = Piece
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
            for p in &board[(row << 3)..((row + 1) << 3)] {
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
        return self.to_board_representation().into_iter();
    }
}

impl Board {
    fn to_board_representation(&self) -> [Piece; 64] {
        let mut board: [Piece; 64] = [Piece::Empty; 64];

        for index in 0..self.pieces.len() {
            let mut v = self.pieces[index];
            while v.leading_zeros() != u64::BITS {
                let first_bit = 63 - v.leading_zeros();
                v = v & !(1 << first_bit);
                board[first_bit as usize] = (index as u8).try_into().unwrap();
            }
        }
        return board;
    }

    pub fn new() -> Self {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        return fen::create_board(&fen).unwrap();
    }

    pub fn piece_at(&self, pos: Square) -> Piece {
        let mask = 1u64 << pos.index();
        for (i, p) in self.pieces.iter().enumerate() {
            if p & mask != 0 {
                return (i as u8).try_into().unwrap();
            }
        }
        // Only reachable if board is invalid
        return Piece::Empty;
    }

    pub fn is_white_to_move(&self) -> bool {
        return self.white_to_move;
    }
}
