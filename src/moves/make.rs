use crate::{
    error::{BoardError, ErrorKind},
    moves::{Move, MoveState},
    piece::Piece,
    position::Position,
    Board,
};

impl Board {
    pub fn make(&mut self, mv: Move) -> Result<(), BoardError> {
        let piece = self.piece_at(mv.origin);
        if piece.is_white() != self.white_to_move {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Attempted to move wrong color",
            ));
        }
        let pseudolegal_moves = self.moves_by_piece(piece);
        if !pseudolegal_moves.contains(&mv) {
            return Err(BoardError::new(ErrorKind::InvalidInput, "Invalid move"));
        }
        // Castle over/out of check
        if piece == Piece::King(piece.is_white())
            && mv.dest.index().abs_diff(mv.origin.index()) == 2
            && (self.is_attacked(
                Position::try_from(mv.dest.index().max(mv.origin.index()) - 1).unwrap(),
                !piece.is_white(),
            ) || self.is_attacked(mv.origin, !piece.is_white()))
        {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Castle over check",
            ));
        }
        //Move is valid, maybe not legal

        let is_ep = if let Some(e) = self.ep_target {
            e.index() == mv.dest.index() && piece == Piece::Pawn(self.white_to_move)
        } else {
            false
        };

        let capture = if is_ep {
            Piece::Pawn(!self.white_to_move)
        } else {
            self.piece_at(mv.dest)
        };
        let move_state = MoveState {
            mv,
            capture,
            castle: self.castle.clone(),
            halfmove: self.halfmove,
            ep_target: self.ep_target.clone(),
        };
        self.move_piece(piece, mv.origin.index(), mv.dest.index());
        self.move_piece(Piece::Empty, mv.dest.index(), mv.origin.index());
        // Clearing Capture
        self.pieces[capture.index()] &= !(1 << mv.dest.index());

        if mv.promotion != Piece::Empty {
            self.pieces[piece.index()] &= !(1 << mv.dest.index());
            self.pieces[mv.promotion.index()] |= 1 << mv.dest.index();
        }

        if is_ep {
            let index = (mv.origin.index() & !0b111) | (mv.dest.index() & 0b111);
            self.pieces[capture.index()] &= !(1 << index);
            self.pieces[Piece::Empty.index()] |= 1 << index;
        }

        let is_castle = Piece::King(piece.is_white()) == piece
            && mv.dest.index().abs_diff(mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && mv.dest.index() > mv.origin.index();
        if is_castle {
            let rook = Piece::Rook(piece.is_white());
            let r_origin = if is_ks_castle {
                mv.origin.index() | 0b111
            } else {
                mv.origin.index() & !0b111
            };
            let r_dest = if is_ks_castle {
                r_origin - 2
            } else {
                r_origin + 3
            };
            self.move_piece(rook, r_origin, r_dest);
            self.move_piece(Piece::Empty, r_dest, r_origin);
        }

        // Updating metadata
        self.move_history.push(move_state);
        self.white_to_move = !self.white_to_move;
        if !piece.is_white() {
            self.fullmove += 1
        }
        if let Piece::Empty = capture {
            self.halfmove += 1;
        } else {
            self.halfmove = 0;
        }

        if let Piece::Pawn(_) = piece {
            //reset halfmove
            self.halfmove = 0;

            // Check if double push to set ep_target
            if mv.dest.index().abs_diff(mv.origin.index()) == 16 {
                self.ep_target =
                    Some(Position::try_from(mv.origin.index().max(mv.dest.index()) - 8).unwrap());
            } else {
                self.ep_target = None;
            }
        } else {
            self.ep_target = None;
        }

        // Update castling
        if piece == Piece::King(piece.is_white()) {
            let ci_offset = if piece.is_white() { 0 } else { 2 };
            self.castle[0 | ci_offset] = false;
            self.castle[1 | ci_offset] = false;
        }
        if let Piece::Rook(is_white) = piece {
            let ci_offset = if is_white { 0 } else { 2 };
            if is_white && mv.origin.index() == 56 || !is_white && mv.origin.index() == 0 {
                self.castle[1 | ci_offset] = false;
            } else if is_white && mv.origin.index() == 63 || !is_white && mv.origin.index() == 7 {
                self.castle[0 | ci_offset] = false;
            }
        }
        if let Piece::Rook(is_white) = capture {
            let ci_offset = if capture.is_white() { 0 } else { 2 };
            if is_white && mv.dest.index() == 56 || !is_white && mv.dest.index() == 0 {
                self.castle[1 | ci_offset] = false;
            } else if is_white && mv.dest.index() == 63 || !is_white && mv.dest.index() == 7 {
                self.castle[0 | ci_offset] = false;
            }
        }
        self.increment_hash(move_state, piece);

        // check if king is in check
        let king: Position = (63
            - self.pieces[Piece::King(piece.is_white()).index()].leading_zeros() as u8)
            .try_into()
            .unwrap();
        if self.is_attacked(king, !piece.is_white()) {
            self.unmake();
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Moving into check",
            ));
        }
        return Ok(());
    }

    pub fn unmake(&mut self) {
        let ms = match self.move_history.pop() {
            Some(m) => m,
            None => return,
        };
        let piece = if ms.mv.promotion == Piece::Empty {
            self.piece_at(ms.mv.dest)
        } else {
            Piece::Pawn(ms.mv.promotion.is_white())
        };
        self.increment_hash(ms, piece);

        self.move_piece(piece, ms.mv.dest.index(), ms.mv.origin.index());
        self.move_piece(Piece::Empty, ms.mv.origin.index(), ms.mv.dest.index());

        if ms.mv.promotion != Piece::Empty {
            self.pieces[ms.mv.promotion.index()] &= !(1 << ms.mv.dest.index());
        }

        let is_ep = if let Some(e) = ms.ep_target {
            e.index() == ms.mv.dest.index() && piece == Piece::Pawn(!self.white_to_move)
        } else {
            false
        };

        if is_ep {
            let bit_index = ((ms.mv.origin.index() >> 3) << 3) | (ms.mv.dest.index() & 0b111);
            self.pieces[ms.capture.index()] |= 1u64 << bit_index;
            self.pieces[Piece::Empty.index()] &= !(1u64 << bit_index);
        } else {
            self.pieces[Piece::Empty.index()] &= !(1 << ms.mv.dest.index());
            self.pieces[ms.capture.index()] |= 1 << ms.mv.dest.index();
        };

        let is_castle = Piece::King(piece.is_white()) == piece
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() > ms.mv.origin.index();
        if is_castle {
            let rook = Piece::Rook(piece.is_white());
            let r_origin = if is_ks_castle {
                ms.mv.origin.index() | 0b111
            } else {
                ms.mv.origin.index() & !0b111
            };
            let r_dest = if is_ks_castle {
                r_origin - 2
            } else {
                r_origin + 3
            };
            self.move_piece(rook, r_dest, r_origin);
            self.move_piece(Piece::Empty, r_origin, r_dest);
        }

        // Resetting metadata
        self.white_to_move = !self.white_to_move;
        self.castle = ms.castle;
        self.ep_target = ms.ep_target;
        self.halfmove = ms.halfmove;
    }

    fn move_piece(&mut self, piece: Piece, from: u8, to: u8) {
        let origin_map = 1 << from;
        let dest_map = 1 << to;
        self.pieces[piece.index()] &= !origin_map;
        self.pieces[piece.index()] |= dest_map;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{moves::Move, piece::Piece, Board};

    impl Board {
        fn is_valid(&self) -> bool {
            let white_pieces = self.team_pieces(true);
            let black_pieces = self.team_pieces(false);
            let empty = self.pieces[Piece::Empty.index()];
            return white_pieces & black_pieces == 0
                && white_pieces & empty == 0
                && black_pieces & empty == 0
                && white_pieces | black_pieces | empty == u64::MAX;
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
        // println!("Possible Moves");
        // for m in board.pseudolegal_moves(board.white_to_move) {
        //     print!("{m}, ");
        // }
        // println!();
        // println!(
        //     "{} possibilities",
        //     board.pseudolegal_moves(board.white_to_move).len()
        // );
        for mv in mvs.iter().rev() {
            board.unmake();
            println!("Board after unmaking {}:\n{}", mv, board);
            println!("Board is valid: {}", board.is_valid());
        }
        println!("Board matches fen:\nNew: {}\nOld: {}", board.to_fen(), fen);
    }
}
