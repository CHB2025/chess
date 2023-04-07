use crate::{
    dir::Dir,
    error::{BoardError, ErrorKind},
    move_gen,
    moves::{Move, MoveState},
    piece::{Color, Piece, PieceKind},
    square::Square,
    Board, Castle,
};

impl Board {
    pub fn make(&mut self, mv: Move) -> Result<(), BoardError> {
        let piece = self[mv.origin];
        if !piece.is_color(self.color_to_move()) {
            // Piece is not empty and matches color
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Attempted to move wrong color",
            ));
        }
        //let moves = self.moves_for_square(mv.origin);
        let moves = move_gen::for_square(self, mv.origin);
        if !moves.contains(&mv) {
            return Err(BoardError::new(ErrorKind::InvalidInput, "Invalid move"));
        }
        //Move is valid, and legal
        unsafe {
            self.make_unchecked(mv);
        }

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
        //self.update_hash(ms);

        self.modify(|b| {
            if ms.mv.promotion != Piece::Empty {
                b.put(piece, ms.mv.dest);
            }
            b.move_replace(ms.mv.dest, ms.mv.origin, ms.capture);

            let is_ep = if let Some(e) = ms.ep_target {
                e.index() == ms.mv.dest.index() && piece.is_kind(PieceKind::Pawn)
            } else {
                false
            };

            if is_ep {
                let bit_index = (ms.mv.origin.index() & !0b111) | (ms.mv.dest.index() & 0b111);
                let sqr = Square(bit_index);
                b.put(ms.capture, sqr);
                b.clear(ms.mv.dest);
            }

            let is_castle = piece.is_kind(PieceKind::King)
                && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
            let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
            if is_castle {
                let r_origin = Square(if is_ks_castle {
                    ms.mv.origin.index() & !0b111
                } else {
                    ms.mv.origin.index() | 0b111
                });
                let r_dest = Square(if is_ks_castle {
                    r_origin.index() as i32 + 2 * Dir::West.offset()
                } else {
                    r_origin.index() as i32 + 3 * Dir::East.offset()
                } as u8);
                b.r#move(r_dest, r_origin);
            }
            b.toggle_color_to_move();
            // Reset hash-tracked metadata
            b.set_castle(Color::White, ms.castle[Color::White]);
            b.set_castle(Color::Black, ms.castle[Color::Black]);
            b.set_ep_target(ms.ep_target);
        });

        // Resetting other metadata
        if piece.is_color(Color::Black) {
            self.fullmove -= 1;
        }
        self.halfmove = ms.halfmove;
    }

    pub unsafe fn make_unchecked(&mut self, mv: Move) {
        let piece = self[mv.origin];

        let ms = self.modify(|b| -> MoveState {
            let is_ep = if let Some(e) = b.board().ep_target {
                e == mv.dest && piece.is_kind(PieceKind::Pawn)
            } else {
                false
            };

            let mut capture = b.r#move(mv.origin, mv.dest);
            if is_ep {
                let index = Square((mv.origin.index() & !0b111) | (mv.dest.index() & 0b111));
                capture = b.clear(index)
            }
            let move_state = MoveState {
                mv,
                capture,
                castle: b.board().castle,
                halfmove: b.board().halfmove,
                ep_target: b.board().ep_target,
            };
            if mv.promotion != Piece::Empty {
                b.put(mv.promotion, mv.dest);
            }
            let is_castle =
                piece.is_kind(PieceKind::King) && mv.dest.index().abs_diff(mv.origin.index()) == 2;
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
                b.r#move(Square(r_origin), Square(r_dest));
            }
            b.toggle_color_to_move();

            if piece.is_kind(PieceKind::Pawn) {
                // Check if double push to set ep_target
                if mv.dest.index().abs_diff(mv.origin.index()) == 16 {
                    b.set_ep_target(Some(Square(mv.origin.index().max(mv.dest.index()) - 8)));
                } else {
                    b.set_ep_target(None);
                }
            } else {
                b.set_ep_target(None);
            }

            // Update castling
            match piece {
                Piece::Filled(PieceKind::King, color) => b.set_castle(color, Castle::None),
                Piece::Filled(PieceKind::Rook, color) => {
                    let is_white = color == Color::White;
                    if is_white && mv.origin.index() == 63 || !is_white && mv.origin.index() == 7 {
                        b.set_castle(color, b.board().castle[color].with_queen_side(false));
                    } else if is_white && mv.origin.index() == 56
                        || !is_white && mv.origin.index() == 0
                    {
                        b.set_castle(color, b.board().castle[color].with_king_side(false));
                    }
                }
                _ => (),
            }
            //if let Piece::Filled(PieceKind::King, color) = piece {
            //    b.set_castle(color, Castle::None);
            //}
            //if let Piece::Filled(PieceKind::Rook, color) = piece {
            //    let is_white = color == Color::White;
            //    if is_white && mv.origin.index() == 63 || !is_white && mv.origin.index() == 7 {
            //        b.set_castle(color, b.board().castle[color].with_queen_side(false));
            //    } else if is_white && mv.origin.index() == 56 || !is_white && mv.origin.index() == 0
            //    {
            //        b.set_castle(color, b.board().castle[color].with_king_side(false));
            //    }
            //}
            if let Piece::Filled(PieceKind::Rook, color) = capture {
                let is_white = capture.is_color(Color::White);
                //let ci_offset = if is_white { 0 } else { 2 };
                if is_white && mv.dest.index() == 63 || !is_white && mv.dest.index() == 7 {
                    b.set_castle(color, b.board().castle[color].with_queen_side(false));
                } else if is_white && mv.dest.index() == 56 || !is_white && mv.dest.index() == 0 {
                    b.set_castle(color, b.board().castle[color].with_king_side(false));
                }
            }
            move_state
        });

        // Updating metadata
        self.move_history.push(ms);
        if piece.is_color(Color::Black) {
            self.fullmove += 1
        }
        if ms.capture == Piece::Empty && piece.is_kind(PieceKind::Pawn) {
            self.halfmove += 1;
        } else {
            self.halfmove = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::piece::Color;
    use crate::ALL;
    use crate::{moves::Move, piece::Piece, Board};

    impl Board {
        fn is_valid(&self) -> bool {
            let white_pieces = self[Color::White];
            let black_pieces = self[Color::Black];
            let empty = self[Piece::Empty];
            (white_pieces & black_pieces).is_empty()
                && (white_pieces & empty).is_empty()
                && (black_pieces & empty).is_empty()
                && white_pieces | black_pieces | empty == ALL
        }
    }

    #[test]
    fn test_make() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp2ppp/3p4/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3")
                .unwrap();
        let _mv = board.legal_moves()[0];
        let m = Move::from_str("d5e6").unwrap();
        board.make(m).unwrap();
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/ppp2ppp/3pP3/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3"
        );
        assert!(board.is_valid());
    }

    #[test]
    fn test_move_sequence() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let mut board = Board::from_fen(fen).unwrap();
        let mvs = ["d7c8q", "b8a6", "c4a6"];
        for mv in mvs {
            let m = Move::from_str(mv).unwrap();
            board.make(m).unwrap();
            assert!(board.is_valid());
        }
        for _mv in mvs.iter().rev() {
            board.unmake();
            assert!(board.is_valid());
        }
    }
}
