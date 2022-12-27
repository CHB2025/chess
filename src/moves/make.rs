use crate::{
    dir::Dir,
    error::{BoardError, ErrorKind},
    moves::{Move, MoveState},
    piece::{Color, Piece, PieceKind},
    square::Square,
    Board,
};

impl Board {
    pub fn make<'a>(&'a mut self, mv: Move) -> Result<(), BoardError> {
        let piece = self[mv.origin];
        if piece.color() != Some(self.color_to_move()) {
            // Piece is not empty and matches color
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Attempted to move wrong color",
            ));
        }
        let moves = self.moves_for_square(mv.origin);
        if !moves.contains(&mv) {
            return Err(BoardError::new(ErrorKind::InvalidInput, "Invalid move"));
        }
        //Move is valid, and legal

        let capture = self
            .position
            .transaction(|t| -> Result<Piece, BoardError> {
                let is_ep = if let Some(e) = self.ep_target {
                    e == mv.dest && piece.kind() == Some(PieceKind::Pawn)
                } else {
                    false
                };

                let mut capture = t.r#move(mv.origin, mv.dest);
                if is_ep {
                    capture = Piece::pawn(!t.position().color_to_move());
                    let index = (mv.origin.index() & !0b111) | (mv.dest.index() & 0b111);
                    t.clear(index.try_into()?);
                }
                if mv.promotion != Piece::Empty {
                    t.put(mv.promotion, mv.dest);
                }
                let is_castle = piece.kind() == Some(PieceKind::King)
                    && mv.dest.index().abs_diff(mv.origin.index()) == 2;
                let is_ks_castle: bool = is_castle && mv.dest.index() < mv.origin.index();
                if is_castle {
                    let r_origin = if is_ks_castle {
                        mv.origin.index() & !0b111
                    } else {
                        mv.origin.index() | 0b111
                    };
                    let r_dest = if is_ks_castle {
                        r_origin as i32 + 2 * Dir::West.offset()
                    } else {
                        r_origin as i32 + 3 * Dir::East.offset()
                    } as u8;
                    t.r#move(r_origin.try_into()?, r_dest.try_into()?);
                }
                Ok(capture)
            })?;

        let move_state = MoveState {
            mv,
            capture,
            castle: self.castle,
            halfmove: self.halfmove,
            ep_target: self.ep_target,
        };

        // Updating metadata
        self.move_history.push(move_state);
        if piece.color() == Some(Color::Black) {
            self.fullmove += 1
        }
        if let Piece::Empty = capture {
            self.halfmove += 1;
        } else {
            self.halfmove = 0;
        }

        if let Piece::Filled(PieceKind::Pawn, _) = piece {
            //reset halfmove
            self.halfmove = 0;

            // Check if double push to set ep_target
            if mv.dest.index().abs_diff(mv.origin.index()) == 16 {
                self.ep_target = Some(Square(mv.origin.index().max(mv.dest.index()) - 8));
            } else {
                self.ep_target = None;
            }
        } else {
            self.ep_target = None;
        }

        // Update castling
        if let Piece::Filled(PieceKind::King, color) = piece {
            let ci_offset = if color == Color::White { 0 } else { 2 };
            self.castle[ci_offset] = false;
            self.castle[1 | ci_offset] = false;
        }
        if let Piece::Filled(PieceKind::Rook, color) = piece {
            let ci_offset = if color == Color::White { 0 } else { 2 };
            if color == Color::White && mv.origin.index() == 63
                || color == Color::Black && mv.origin.index() == 7
            {
                self.castle[1 | ci_offset] = false;
            } else if color == Color::White && mv.origin.index() == 56
                || color == Color::Black && mv.origin.index() == 0
            {
                self.castle[ci_offset] = false;
            }
        }
        if let Piece::Filled(PieceKind::Rook, color) = capture {
            let ci_offset = if color == Color::White { 0 } else { 2 };
            if color == Color::White && mv.dest.index() == 63
                || color == Color::Black && mv.dest.index() == 7
            {
                self.castle[1 | ci_offset] = false;
            } else if color == Color::White && mv.dest.index() == 56
                || color == Color::Black && mv.dest.index() == 0
            {
                self.castle[ci_offset] = false;
            }
        }
        self.increment_hash(move_state, piece);
        Ok(())
    }

    pub fn unmake(&mut self) {
        let ms = match self.move_history.pop() {
            Some(m) => m,
            None => return,
        };
        let piece = match ms.mv.promotion {
            Piece::Empty => self[ms.mv.dest],
            Piece::Filled(_, color) => Piece::pawn(color),
        };
        self.increment_hash(ms, piece);

        self.position
            .transaction(|t| -> Result<(), BoardError> {
                if ms.mv.promotion != Piece::Empty {
                    t.put(piece, ms.mv.dest);
                }
                t.move_replace(ms.mv.dest, ms.mv.origin, ms.capture);

                let is_ep = if let Some(e) = ms.ep_target {
                    e.index() == ms.mv.dest.index() && piece.kind() == Some(PieceKind::Pawn)
                } else {
                    false
                };

                if is_ep {
                    let bit_index = (ms.mv.origin.index() & !0b111) | (ms.mv.dest.index() & 0b111);
                    let sqr = Square(bit_index);
                    t.put(ms.capture, sqr);
                    t.clear(ms.mv.dest);
                }

                let is_castle = Some(PieceKind::King) == piece.kind()
                    && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
                let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
                if is_castle {
                    let r_origin: Square = Square(if is_ks_castle {
                        ms.mv.origin.index() & !0b111
                    } else {
                        ms.mv.origin.index() | 0b111
                    });
                    let r_dest: Square = Square(if is_ks_castle {
                        r_origin.index() as i32 + 2 * Dir::West.offset()
                    } else {
                        r_origin.index() as i32 + 3 * Dir::East.offset()
                    } as u8);
                    t.r#move(r_dest, r_origin);
                }
                Ok(())
            })
            .unwrap();

        // Resetting metadata
        if piece.color() == Some(Color::Black) {
            self.fullmove -= 1;
        }
        self.castle = ms.castle;
        self.ep_target = ms.ep_target;
        self.halfmove = ms.halfmove;
    }

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::piece::Color;
    use crate::{moves::Move, piece::Piece, Board};

    impl Board {
        fn is_valid(&self) -> bool {
            let white_pieces = self.position[Color::White];
            let black_pieces = self.position[Color::Black];
            let empty = self.position[Piece::Empty];
            white_pieces & black_pieces == 0
                && white_pieces & empty == 0
                && black_pieces & empty == 0
                && white_pieces | black_pieces | empty == u64::MAX
        }
    }

    #[test]
    fn test_make() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp2ppp/3p4/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3")
                .unwrap();
        let m = Move::from_str("d5e6").unwrap();
        board.make(m).unwrap();
        println!("{}", board);
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/ppp2ppp/3pP3/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3"
        )
    }

    #[test]
    fn test_move_sequence() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let mut board = Board::from_fen(fen).unwrap();
        let mvs = ["d7c8q", "b8a6", "c4a6"];
        println!("Initial Board:\n{}", board);
        for mv in mvs {
            let m = Move::from_str(mv).unwrap();
            if let Err(e) = board.make(m) {
                println!("Error making move: {e}");
                break;
            };
            println!("Board after making move {}:\n{}", m, board);
            println!("Board is valid: {}", board.is_valid());
        }
        for mv in mvs.iter().rev() {
            board.unmake();
            println!("Board after unmaking {}:\n{}", mv, board);
            println!("Board is valid: {}", board.is_valid());
        }
        println!("Board matches fen:\nNew: {}\nOld: {}", board.to_fen(), fen);
    }
}
