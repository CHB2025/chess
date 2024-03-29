mod board;
pub use board::{Board, builder::BoardBuilder};

mod ray;
pub use ray::Ray;

mod square;
pub use square::{Square, squares};

mod moves;
pub use moves::Move;
pub(crate) use moves::MoveState;

mod piece;
pub use piece::{Color, Piece, PieceKind, PROMO_PIECES};

mod error;
pub use error::{BoardError, ErrorKind};

mod dir; //Does dir need to be public?
pub use dir::{Dir, ALL_DIRS};

mod bitboard;
pub use bitboard::{Bitboard, ALL, EMPTY, NOT_A_FILE, NOT_H_FILE};

mod check;
pub use check::Check;

mod castle;
pub use castle::{Castle};

pub mod move_gen;
