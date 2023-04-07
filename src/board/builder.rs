use std::{str::FromStr, ops};

use regex::Regex;

use super::{Board, BoardIter};
use crate::{BoardError, Castle, Check, Color, Dir, ErrorKind, Piece, PieceKind, Square};

#[derive(Debug)]
pub struct BoardBuilder {
    pieces: [Piece; 64],
    color_to_move: Color,
    castle: [Castle; 2],
    ep_target: Option<Square>,
    halfmove: u32,
    fullmove: u32,
}

impl IntoIterator for &BoardBuilder {
    type Item = Piece;
    type IntoIter = BoardIter; 

    /// Returns an iterator of all the pieces on the board in big-endian order
    /// (h8-a1).
    fn into_iter(self) -> Self::IntoIter {
        self.pieces.into_iter()
    }
}

impl Default for BoardBuilder {
    /// Returns a [BoardBuilder] containing the default chess starting position
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            pieces: [
                // Black pieces
                Piece::rook(Color::Black), Piece::knight(Color::Black), Piece::bishop(Color::Black), Piece::king(Color::Black), Piece::queen(Color::Black), Piece::bishop(Color::Black), Piece::knight(Color::Black), Piece::rook(Color::Black),
                // Black Pawns
                Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black), Piece::pawn(Color::Black),
                // Blank rows
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                // White Pawns
                Piece::pawn(Color::White), Piece::pawn(Color::White),Piece::pawn(Color::White),Piece::pawn(Color::White),Piece::pawn(Color::White),Piece::pawn(Color::White),Piece::pawn(Color::White),Piece::pawn(Color::White),
                // Other white pieces
                Piece::rook(Color::White), Piece::knight(Color::White), Piece::bishop(Color::White), Piece::king(Color::White), Piece::queen(Color::White), Piece::bishop(Color::White), Piece::knight(Color::White), Piece::rook(Color::White),
            ],
            castle: [Castle::Both; 2],
            color_to_move: Color::White,
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
        }
    }
}

impl ops::Index<Square> for BoardBuilder {
    type Output = Piece;

    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index.index() as usize]
    }
}
impl ops::IndexMut<Square> for BoardBuilder {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.pieces[index.index() as usize]
    }
}


impl BoardBuilder {
    /// Returns a new [BoardBuilder].
    /// Starts with an empty board with White to move, no castling rights, on
    /// move 1
    pub fn new() -> Self {
        BoardBuilder {
            pieces: [Piece::Empty; 64],
            color_to_move: Color::White,
            castle: [Castle::None; 2],
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    /// Creates a new [BoardBuilder] from a string in Forsynth-Edwards Notation (FEN). Returns a
    /// [BoardError] if the string is improperly formatted.
    ///
    /// # Examples
    /// ```
    /// # use chb_chess::BoardBuilder;
    /// // The starting FEN is valid and properly formatted, so it returns Ok
    /// let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let builder = BoardBuilder::from_fen(starting_fen);
    /// println!("{:?}", builder);
    ///
    /// assert!(builder.is_ok());
    ///
    /// // An invalid FEN will still return Ok if it is properly formatted
    /// // This is useful when starting with a partially assembled FEN and applying changes to it
    /// // which will make it valid
    /// let invalid_fen = "rnbq1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let builder = BoardBuilder::from_fen(invalid_fen);
    ///
    /// assert!(builder.is_ok());
    ///
    /// // An impropperly formatted fen will return an Err
    /// let improper_fen = "rnbqkbnr/pppppppp/8/3i4/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let builder = BoardBuilder::from_fen(improper_fen);
    ///
    /// assert!(builder.is_err());
    ///
    /// // The half and full move counts are optional
    /// let fen_with_no_move_count = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
    /// let builder = BoardBuilder::from_fen(fen_with_no_move_count);
    ///
    /// assert!(builder.is_ok());
    ///
    /// # Ok::<(), chb_chess::BoardError>(());
    /// ```
    pub fn from_fen(fen: &str) -> Result<Self, BoardError> {
        let mut builder = Self::new();
        let short_err = || BoardError::new(ErrorKind::InvalidInput, "Missing sections of FEN");
        let mut sections = fen.split(' ');

        let b = sections.next().ok_or_else(short_err)?;

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
                builder.pieces[square] = p;
                pos_count += 1;
            }
            row_count += 1;
        }
        if row_count != 8 || pos_count != 64 {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Invalid Position Input. Row or Position count did not match expected",
            ));
        }

        builder.color_to_move = sections.next().ok_or_else(short_err)?.parse()?;

        let castling = sections.next().ok_or_else(short_err)?;
        let c_re =
            Regex::new(r"^(?:K?Q?k?q?|-)$").expect("Invalid Regex used to check castling rights");
        if !c_re.is_match(castling) {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Castling rights are invalid",
            ));
        }
        for c in castling.chars() {
            let p: Piece = c.try_into()?;
            match p {
                Piece::Filled(PieceKind::King, color) => {
                    builder.castle[color] = builder.castle[color].with_king_side(true)
                }
                Piece::Filled(PieceKind::Queen, color) => {
                    builder.castle[color] = builder.castle[color].with_queen_side(true)
                }
                _ => (),
            }
        }

        // EP Target
        if let Ok(p) = Square::from_str(sections.next().ok_or_else(short_err)?) {
            builder.ep_target = Some(p);
            // Check rank and if there is a pawn in capture position right above it based on color to move
        }

        // Move counts
        builder.halfmove = match sections.next() {
            Some(hm) => hm.parse()?,
            None => 0,
        };
        builder.fullmove = match sections.next() {
            Some(fm) => fm.parse()?,
            None => 1,
        };

        Ok(builder)
    }

    /// Puts the given [Piece] at the [Square]. Removes the [Piece] that was previously there.
    /// Takes self by mutable refrence and returns it again so it can be chained with other
    /// operations.
    ///
    /// # Examples
    /// ```
    /// # use chb_chess::{BoardBuilder, Piece, BoardError};
    /// let mut builder = BoardBuilder::default();
    /// builder
    ///     .put("p".parse()?, "c5".parse()?)
    ///     .put(Piece::Empty, "c7".parse()?);
    ///
    /// # Ok::<(), BoardError>(())
    /// ```
    pub fn put(&mut self, piece: Piece, square: Square) -> &mut Self {
        self.pieces[square] = piece;
        self
    }

    /// Sets the [Color] which will move first
    pub fn color_to_move(&mut self, color: Color) -> &mut Self {
        self.color_to_move = color;
        self
    }
    /// Set the castling rights for a given side. If the [Piece] provided is not a King or a Queen
    /// the value is ignored
    pub fn castle(&mut self, side: Color, castle: Castle) -> &mut Self {
        self.castle[side] = castle;
        self
    }
    /// Set the fullmove counter
    pub fn fullmove(&mut self, fullmove: u32) -> &mut Self {
        self.fullmove = fullmove;
        self
    }
    /// Set the halfmove counter
    pub fn halfmove(&mut self, halfmove: u32) -> &mut Self {
        self.halfmove = halfmove;
        self
    }

    /// Set(or clear) the En Passant target [Square]
    pub fn ep_target(&mut self, target: Option<Square>) -> &mut Self {
        self.ep_target = target;
        self
    }

    /// Validates everything necessary to ensure that the board can generate things like attacks,
    /// pins, checks. If `Ok`, the board is able to generate and make at least one move.
    ///
    /// Validates:
    /// - Both kings exist
    /// - Castling rights are only given if the king is at the starting position
    /// - The en passant target square is in an appropriate location
    ///
    /// Does not validate:
    /// - The king of the second team to move is not in check. This can lead to undefined
    ///   behavior/panics after one move if the king is captured.
    /// - The first team to move has legal moves. Won't lead to undefined behavior, but why set
    ///   it up.
    fn partial_validate(&self) -> Result<(), BoardError> {
        // Validate that
        // both kings exist
        let king_counts: (u32, u32) = self.pieces.into_iter().fold((0, 0), |(mut w, mut b), p| {
            if let Piece::Filled(kind, color) = p {
                if kind == PieceKind::King {
                    match color {
                        Color::White => w += 1,
                        Color::Black => b += 1,
                    }
                }
            }
            (w, b)
        });
        if (1, 1) != king_counts {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Board must contain one white and one black king",
            ));
        }
        // castling follows the rules
        let w_king = self
            .pieces
            .into_iter()
            .position(|p| p == Piece::king(Color::White))
            .expect("King is guaranteed to be on the board");
        if self.castle[Color::White] != Castle::None && w_king != 59 {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "White king may not castle if it is not at e1",
            ));
        }
        let b_king = self
            .pieces
            .into_iter()
            .position(|p| p == Piece::king(Color::Black))
            .expect("King is guaranteed to be on the board");
        if self.castle[Color::Black] != Castle::None && b_king != 3 {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Black king may not castle if it is not at e8",
            ));
        }
        // ep_target is possible
        if let Some(target) = self.ep_target {
            let (allowed_rank, pawn_dir) = match self.color_to_move {
                Color::White => (2, Dir::South),
                Color::Black => (5, Dir::North),
            };
            if target.rank() != allowed_rank
                || self.pieces[target
                    .checked_add(pawn_dir)
                    .expect("on allowed rank so it can add North/South")]
                    != Piece::pawn(!self.color_to_move)
            {
                return Err(BoardError::new(
                    ErrorKind::InvalidInput,
                    "Illegal En Passant target",
                ));
            }
        }

        Ok(())
    }

    /// Builds a board using the current state of the BoardBuilder. If the current state is not
    /// valid, it will return a BoardError with a message describing the issue.
    ///
    /// # Examples
    /// ```
    /// # use chb_chess::{BoardBuilder, Piece, Square};
    ///
    /// let mut builder = BoardBuilder::default();
    ///
    /// assert!(builder.build().is_ok());
    ///
    /// // Replacing the white king to invalidate the BoardBuilder
    /// builder.put(Piece::Empty, "e1".parse::<Square>()?);
    ///
    /// assert!(builder.build().is_err());
    /// # Ok::<(), chb_chess::BoardError>(())
    /// ```
    pub fn build(&self) -> Result<Board, BoardError> {
        self.partial_validate()?;

        let mut board = Board::empty();
        // color to move is opposite of what it should be so we can check if
        // the current opposing king is in check. Board is invalid if it is
        board.halfmove = self.halfmove;
        board.fullmove = self.fullmove;

        board.modify(|m| {
            m.set_ep_target(self.ep_target);
            m.set_castle(Color::White, self.castle[Color::White]);
            m.set_castle(Color::Black, self.castle[Color::Black]);
            if m.board.color_to_move() == self.color_to_move {
                m.toggle_color_to_move();
            }
            for (sq, p) in self.pieces.iter().enumerate() {
                m.put(*p, sq.try_into().expect("will be valid square"));
            }
        });
        if board.check != Check::None {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "First color to move may not be able to capture the opposing king",
            ));
        }
        // Switching color to move and updating attacks, pins, and checks
        board.modify(|m| (m.toggle_color_to_move()));

        if board.legal_moves().len() == 0 {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "First color to move must have available moves",
            ));
        }
        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use crate::BoardBuilder;

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
            let builder = BoardBuilder::from_fen(&fen);
            assert_eq!(
                builder.is_ok() && builder.unwrap().build().is_ok(),
                is_valid,
                "Testing {}",
                fen
            );
        }
    }

    #[test]
    fn test_create_board() {
        let fens = valid_fens();

        for f in fens {
            let game = BoardBuilder::from_fen(&f)
                .expect("Failed to create builder")
                .build()
                .expect("Failed to create game");
            println!("{}", game);
            assert_eq!(f, game.to_fen());
        }
    }
}
