use crate::{
    error::{BoardError, ErrorKind},
    hash::{hash_index, MAX_PIECE_INDEX},
    moves::{Move, MoveState},
    piece::Piece,
    square::Square,
    Board,
};

use super::generate;

impl Board {
    pub fn make(&mut self, mv: Move) -> Result<(), BoardError> {
        let piece = self[mv.origin];
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
                Square::try_from(mv.dest.index().max(mv.origin.index()) - 1)?,
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
            self[mv.dest]
        };
        let move_state = MoveState {
            mv,
            capture,
            castle: self.castle,
            halfmove: self.halfmove,
            ep_target: self.ep_target,
        };
        self.position.r#move(mv.origin, mv.dest);
        if mv.promotion != Piece::Empty {
            self.position.put(mv.promotion, mv.dest);
        }

        if is_ep {
            let index = (mv.origin.index() & !0b111) | (mv.dest.index() & 0b111);
            self.position.clear(index.try_into().unwrap());
        }

        let is_castle = Piece::King(piece.is_white()) == piece
            && mv.dest.index().abs_diff(mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && mv.dest.index() < mv.origin.index();
        if is_castle {
            let r_origin = if is_ks_castle {
                mv.origin.index() & !0b111
            } else {
                mv.origin.index() | 0b111
            };
            let r_dest = if is_ks_castle {
                r_origin as i32 + 2 * generate::LEFT
            } else {
                r_origin as i32 + 3 * generate::RIGHT
            } as u8;
            self.position
                .r#move(r_origin.try_into().unwrap(), r_dest.try_into().unwrap());
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
                    Some(Square::try_from(mv.origin.index().max(mv.dest.index()) - 8).unwrap());
            } else {
                self.ep_target = None;
            }
        } else {
            self.ep_target = None;
        }

        // Update castling
        if let Piece::King(is_white) = piece {
            let ci_offset = if is_white { 0 } else { 2 };
            self.castle[ci_offset] = false;
            self.castle[1 | ci_offset] = false;
        }
        if let Piece::Rook(is_white) = piece {
            let ci_offset = if is_white { 0 } else { 2 };
            if is_white && mv.origin.index() == 63 || !is_white && mv.origin.index() == 7 {
                self.castle[1 | ci_offset] = false;
            } else if is_white && mv.origin.index() == 56 || !is_white && mv.origin.index() == 0 {
                self.castle[ci_offset] = false;
            }
        }
        if let Piece::Rook(is_white) = capture {
            let ci_offset = if capture.is_white() { 0 } else { 2 };
            if is_white && mv.dest.index() == 63 || !is_white && mv.dest.index() == 7 {
                self.castle[1 | ci_offset] = false;
            } else if is_white && mv.dest.index() == 56 || !is_white && mv.dest.index() == 0 {
                self.castle[ci_offset] = false;
            }
        }
        self.increment_hash(move_state, piece);

        // check if king is in check
        // What if there is no king on the board? Will that case ever arise?
        // Leaving unwrap to panic if this ever happens
        let king: Square = (63
            - self.position[Piece::King(piece.is_white())].leading_zeros() as u8)
            .try_into()
            .unwrap();
        if self.is_attacked(king, !piece.is_white()) {
            self.unmake();
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Moving into check",
            ));
        }
        Ok(())
    }

    pub fn unmake(&mut self) {
        let ms = match self.move_history.pop() {
            Some(m) => m,
            None => return,
        };
        let piece = if ms.mv.promotion == Piece::Empty {
            self[ms.mv.dest]
        } else {
            Piece::Pawn(ms.mv.promotion.is_white())
        };
        self.increment_hash(ms, piece);
        if ms.mv.promotion != Piece::Empty {
            self.position.put(piece, ms.mv.dest);
        }
        self.position.move_replace(ms.mv.dest, ms.mv.origin, ms.capture);

        

        let is_ep = if let Some(e) = ms.ep_target {
            e.index() == ms.mv.dest.index() && piece == Piece::Pawn(!self.white_to_move)
        } else {
            false
        };

        if is_ep {
            let bit_index = ((ms.mv.origin.index() >> 3) << 3) | (ms.mv.dest.index() & 0b111);
            let sqr = Square::try_from(bit_index).expect("Invalid index for ep capture pawn");
            self.position.put(ms.capture, sqr);
            self.position.clear(ms.mv.dest);
        }

        let is_castle = Piece::King(piece.is_white()) == piece
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
        if is_castle {
            let r_origin: Square = if is_ks_castle {
                ms.mv.origin.index() & !0b111
            } else {
                ms.mv.origin.index() | 0b111
            }
            .try_into()
            .expect("Invalid index for castling rook");
            let r_dest: Square = (if is_ks_castle {
                r_origin.index() as i32 + 2 * generate::LEFT
            } else {
                r_origin.index() as i32 + 3 * generate::RIGHT
            } as u8)
                .try_into()
                .expect("Invalid dest index for castling rook");
            self.position.r#move(r_dest, r_origin);
        }

        // Resetting metadata
        self.white_to_move = !self.white_to_move;
        self.castle = ms.castle;
        self.ep_target = ms.ep_target;
        self.halfmove = ms.halfmove;
    }

    fn increment_hash(&mut self, ms: MoveState, p: Piece) {
        self.hash ^= self.hash_keys[hash_index(p, ms.mv.origin.index().into())];
        if ms.mv.promotion == Piece::Empty {
            self.hash ^= self.hash_keys[hash_index(p, ms.mv.dest.index().into())];
        } else {
            self.hash ^= self.hash_keys[hash_index(ms.mv.promotion, ms.mv.dest.index().into())];
        }

        let is_ep = if let Some(pos) = ms.ep_target {
            pos == ms.mv.dest && p == Piece::Pawn(p.is_white())
        } else {
            false
        };

        if ms.capture != Piece::Empty && !is_ep {
            self.hash ^= self.hash_keys[hash_index(ms.capture, ms.mv.dest.index().into())];
        }
        if is_ep {
            let index = (ms.mv.origin.index() & !0b111) | (ms.mv.dest.index() & 0b111);
            self.hash ^= self.hash_keys[hash_index(ms.capture, index.into())];
        }

        let is_castle = Piece::King(p.is_white()) == p
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
        if is_castle {
            let rook = Piece::Rook(p.is_white());
            let r_origin = if is_ks_castle {
                ms.mv.origin.index() & !0b111
            } else {
                ms.mv.origin.index() | 0b111
            };
            let r_dest = if is_ks_castle {
                r_origin + 2
            } else {
                r_origin - 3
            };
            self.hash ^= self.hash_keys[hash_index(rook, r_origin.into())];
            self.hash ^= self.hash_keys[hash_index(rook, r_dest.into())];
        }

        let mut next_index = MAX_PIECE_INDEX + 1;
        self.hash ^= self.hash_keys[next_index];
        next_index += 1;

        for (i, c) in self.castle.iter().enumerate() {
            if *c != ms.castle[i] {
                self.hash ^= self.hash_keys[next_index];
            }
            next_index += 1;
        }
        if let Some(pos) = ms.ep_target {
            let col = (pos.index() & 0b111) as usize;
            self.hash ^= self.hash_keys[next_index + col - 1];
        }
        if let Some(pos) = self.ep_target {
            let col = (pos.index() & 0b111) as usize;
            self.hash ^= self.hash_keys[next_index + col - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{moves::Move, piece::Piece, Board};

    impl Board {
        fn is_valid(&self) -> bool {
            let white_pieces = self.position.team_pieces(true);
            let black_pieces = self.position.team_pieces(false);
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
