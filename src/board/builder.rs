use super::Board;
use crate::{BoardError, Color, Dir, ErrorKind, Piece, PieceKind, Square};

pub struct BoardBuilder {
    pieces: [Piece; 64],
    color_to_move: Color,
    castle: [bool; 4],
    ep_target: Option<Square>,
    halfmove: u32,
    fullmove: u32,
}

impl Default for BoardBuilder {
    /// Returns a BoardBuilder containing the default chess starting position
    fn default() -> Self {
        Self {
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
        }
    }
}

impl BoardBuilder {
    /// Returns a new BoardBuilder.
    /// Starts with an empty board with White to move, no castling rights, on
    /// move 1
    pub fn new() -> Self {
        BoardBuilder {
            pieces: [Piece::Empty; 64],
            color_to_move: Color::White,
            castle: [false; 4],
            ep_target: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    /// Puts the given piece at the square. Removes the piece that was previously there.
    pub fn put(mut self, piece: Piece, square: Square) -> Self {
        self.pieces[square] = piece;
        self
    }

    /// Sets the color which will move first
    pub fn color_to_move(mut self, color: Color) -> Self {
        self.color_to_move = color;
        self
    }
    /// Set the castling rights for a given side. If the Piece provided is not a King or a Queen
    /// the value is ignored
    pub fn castle(mut self, side: Piece, can_castle: bool) -> Self {
        if let Piece::Filled(kind, color) = side {
            let offset = if color == Color::White { 0 } else { 2 };
            match kind {
                PieceKind::King => self.castle[offset] = can_castle,
                PieceKind::Queen => self.castle[offset + 1] = can_castle,
                _ => (),
            }
        }
        self
    }
    /// Set the fullmove counter
    pub fn fullmove(mut self, fullmove: u32) -> Self {
        self.fullmove = fullmove;
        self
    }
    /// Set the halfmove counter
    pub fn halfmove(mut self, halfmove: u32) -> Self {
        self.halfmove = halfmove;
        self
    }

    /// Set(or clear) the En Passant target square
    pub fn ep_target(mut self, target: Option<Square>) -> Self {
        self.ep_target = target;
        self
    }

    /// Validates everything necessary to ensure that the board can generate things like attacks,
    /// pins, checks.
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
        if (1, 1) == king_counts {
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
        if (self.castle[0] | self.castle[1]) && w_king != 59 {
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
        if (self.castle[0] | self.castle[1]) && b_king != 3 {
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

    pub fn build(&self) -> Result<Board, BoardError> {
        self.partial_validate()?;
        todo!()
    }
}
