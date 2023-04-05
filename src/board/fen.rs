use core::fmt;
use std::{fmt::Display, str::FromStr};

use crate::{Board, BoardBuilder, BoardError, Piece};

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

impl Board {
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

        if self.castle.iter().all(|x| !x) {
            output += " -";
        } else {
            output.push(' ');
            for (i, &can_castle) in self.castle.iter().enumerate() {
                let side = Piece::try_from((i & 1) << 1 | (i >> 1)).unwrap();
                if can_castle {
                    output += &format!("{}", side);
                }
            }
        }

        if let Some(ept) = self.ep_target {
            output += &format!(" {}", ept)
        } else {
            output += " -"
        }

        output += &format!(" {}", self.halfmove);
        output += &format!(" {}", self.fullmove);

        output
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
}

#[cfg(test)]
mod tests {
    use crate::Board;

    fn valid_fens() -> [String; 6] {
        [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(),
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(),
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".to_string(),
        ]
    }

    #[test]
    fn test_is_valid() {
        let fen_strings = [
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                true,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PP2PPPPP/RNBQKBNR w KQkq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQfdskq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR p KQkq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - -324 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 -219",
                false,
            ),
        ];
        for (fen, is_valid) in fen_strings {
            assert_eq!(!Board::from_fen(fen).is_err(), is_valid, "Testing {}", fen);
        }
    }

    #[test]
    fn test_create_board() {
        let fens = valid_fens();

        for f in fens {
            let game = Board::from_fen(&f).expect("Failed to create game");
            println!("{}", game);
            assert_eq!(f, game.to_fen());
        }
    }
}
