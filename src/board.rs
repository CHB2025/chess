use std::{
    default,
    fmt::{self, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{
    move_gen, Bitboard, BoardBuilder, BoardError, Castle, Check, Color, Move, MoveState, Piece,
    Ray, Square, ALL, EMPTY,
};

use self::modify::Modifier;

mod attacks;
pub mod builder;
mod hash;
mod index;
mod make;
mod modify;
mod perft;

#[derive(Clone)]
pub struct Board {
    bitboards: [Bitboard; 13],
    color_bitboards: [Bitboard; 2],
    attacks: Bitboard,
    pins: Bitboard,
    check: Check,
    color_to_move: Color,
    pieces: [Piece; 64],
    castle: [Castle; 2],
    ep_target: Option<Square>,
    halfmove: u32,
    fullmove: u32,
    move_history: Vec<MoveState>,
    hash: u64,
    hash_keys: [u64; 781],
}

pub type BoardIter = std::array::IntoIter<Piece, 64>;

impl IntoIterator for &Board {
    type Item = Piece;
    type IntoIter = BoardIter;

    /// Returns an iterator of all the pieces on the board in big-endian order
    /// (h8-a1).
    fn into_iter(self) -> Self::IntoIter {
        self.pieces.into_iter()
    }
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

impl FromStr for Board {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_fen(s)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl default::Default for Board {
    fn default() -> Self {
        BoardBuilder::default()
            .build()
            .expect("Default board is valid")
    }
}

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use this function to create a board from a string in Forsynth-Edwards
    /// Notation (FEN). Returns a [BoardError] if the string is invalid.
    ///
    /// # Examples
    /// ```
    /// # use chb_chess::Board;
    ///
    /// let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board = Board::from_fen(starting_fen)?;
    ///
    /// //assert_eq!(board, Board::default());
    ///
    /// # Ok::<(), chb_chess::BoardError>(())
    /// ```
    pub fn from_fen(fen: impl Into<String>) -> Result<Self, BoardError> {
        let f: String = fen.into();
        BoardBuilder::from_fen(&f)?.build()
    }

    fn empty() -> Self {
        Self {
            bitboards: [
                EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
                ALL,
            ],
            color_bitboards: [EMPTY; 2],
            attacks: EMPTY,
            pins: EMPTY,
            check: Check::None,
            pieces: [Piece::Empty; 64],
            color_to_move: Color::White,
            castle: [Castle::None; 2],
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
            move_history: Vec::new(),
            hash: 0,
            hash_keys: hash::zobrist_keys(),
        }
    }

    #[inline]
    fn modify<'a, T>(&'a mut self, arg: impl FnOnce(&mut Modifier<'a>) -> T) -> T {
        let mut action = Modifier { board: self };
        let response = arg(&mut action);
        action.complete();
        response
    }

    /// Creates a representation of the board in Forsynth-Edwards Notation(FEN)
    pub fn to_fen(&self) -> String {
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

        output += &format!(" {}", self.color_to_move);

        output += &match self.castle {
            [Castle::None, Castle::None] => " -".to_owned(),
            [w, Castle::None] if w != Castle::None => format!(" {}", w).to_uppercase(),
            [Castle::None, b] => format!(" {}", b),
            [w, b] => format!(" {}{}", format!("{}", w).to_uppercase(), b),
        };

        if let Some(ept) = self.ep_target {
            output += &format!(" {}", ept)
        } else {
            output += " -"
        }

        output += &format!(" {}", self.halfmove);
        output += &format!(" {}", self.fullmove);

        output
    }

    pub fn is_white_to_move(&self) -> bool {
        self.color_to_move == Color::White
    }
    pub fn color_to_move(&self) -> Color {
        self.color_to_move
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        move_gen::legal(self)
    }

    pub fn king(&self, color: Color) -> Square {
        self[Piece::king(color)]
            .first_square()
            .expect("King is not on the board")
    }

    pub fn pin_on_square(&self, square: Square) -> Option<Ray> {
        let piece = self[square];
        match piece {
            Piece::Filled(_, color) => {
                if self.pins.contains(square) {
                    Ray::from(self.king(color), square)
                } else {
                    None
                }
            }
            Piece::Empty => None,
        }
    }

    pub fn check(&self) -> Check {
        self.check
    }

    pub fn ep_target(&self) -> Option<Square> {
        self.ep_target
    }

    pub fn attacks(&self) -> Bitboard {
        self.attacks
    }

    pub fn pins(&self) -> Bitboard {
        self.pins
    }

    pub fn castle(&self, color: Color) -> Castle {
        self.castle[color]
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}
