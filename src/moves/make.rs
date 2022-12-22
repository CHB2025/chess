use crate::{
    error::{BoardError, ErrorKind},
    hash::{hash_index, MAX_PIECE_INDEX},
    moves::{Move, MoveState},
    piece::{Color, Piece, PieceType},
    square::Square,
    Board, dir::Dir,
};

impl Board {
    pub fn make(&mut self, mv: Move) -> Result<(), BoardError> {
        let piece = self[mv.origin];
        if piece.color() != Some(self.color_to_move) {
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

        let is_ep = if let Some(e) = self.ep_target {
            e == mv.dest && piece == Piece::Filled(PieceType::Pawn, self.color_to_move)
        } else {
            false
        };

        let mut capture = self.position.r#move(mv.origin, mv.dest);
        if is_ep {
            capture = Piece::Filled(PieceType::Pawn, !self.color_to_move);
            let index = Square((mv.origin.index() & !0b111) | (mv.dest.index() & 0b111));
            self.position.clear(index);
        }
        if mv.promotion != Piece::Empty {
            self.position.put(mv.promotion, mv.dest);
        }
        let is_castle = Piece::Filled(PieceType::King, self.color_to_move) == piece
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
            self.position.r#move(Square(r_origin), Square(r_dest));
        }
        let move_state = MoveState {
            mv,
            capture,
            castle: self.castle,
            halfmove: self.halfmove,
            ep_target: self.ep_target,
        };

        // Updating metadata
        self.move_history.push(move_state);
        self.color_to_move = !self.color_to_move;
        if piece.color() == Some(Color::Black) {
            self.fullmove += 1
        }
        if let Piece::Empty = capture {
            self.halfmove += 1;
        } else {
            self.halfmove = 0;
        }

        if let Piece::Filled(PieceType::Pawn, _) = piece {
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
        if let Piece::Filled(PieceType::King, color) = piece {
            let ci_offset = if color == Color::White { 0 } else { 2 };
            self.castle[ci_offset] = false;
            self.castle[1 | ci_offset] = false;
        }
        if let Piece::Filled(PieceType::Rook, color) = piece {
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
        if let Piece::Filled(PieceType::Rook, color) = capture {
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
            Piece::Filled(_, color) => Piece::Filled(PieceType::Pawn, color),
        };
        self.increment_hash(ms, piece);
        if ms.mv.promotion != Piece::Empty {
            self.position.put(piece, ms.mv.dest);
        }
        self.position
            .move_replace(ms.mv.dest, ms.mv.origin, ms.capture);

        let is_ep = if let Some(e) = ms.ep_target {
            e.index() == ms.mv.dest.index() && piece.piece_type() == Some(PieceType::Pawn)
        } else {
            false
        };

        if is_ep {
            let bit_index = ((ms.mv.origin.index() >> 3) << 3) | (ms.mv.dest.index() & 0b111);
            let sqr = Square(bit_index);
            self.position.put(ms.capture, sqr);
            self.position.clear(ms.mv.dest);
        }

        let is_castle = Some(PieceType::King) == piece.piece_type()
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
            self.position.r#move(r_dest, r_origin);
        }

        // Resetting metadata
        if self.color_to_move == Color::White {
            self.fullmove -= 1;
        }
        self.color_to_move = !self.color_to_move;
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
            pos == ms.mv.dest && p.piece_type() == Some(PieceType::Pawn)
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

        let is_castle = Some(PieceType::King) == p.piece_type()
            && ms.mv.dest.index().abs_diff(ms.mv.origin.index()) == 2;
        let is_ks_castle: bool = is_castle && ms.mv.dest.index() < ms.mv.origin.index();
        if is_castle {
            let rook = Piece::Filled(PieceType::Rook, !self.color_to_move);
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
        // Changing sides
        self.hash ^= self.hash_keys[next_index];
        next_index += 1;

        for i in 0..4 {
            if self.castle[i] != ms.castle[i] {
                self.hash ^= self.hash_keys[next_index];
            } 
            next_index += 1;
        }
        if let Some(pos) = ms.ep_target {
            self.hash ^= self.hash_keys[next_index + pos.file() as usize];
        }
        if let Some(pos) = self.ep_target {
            self.hash ^= self.hash_keys[next_index + pos.file() as usize]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::piece::Color;
    use crate::{moves::Move, piece::Piece, Board};

    impl Board {
        fn is_valid(&self) -> bool {
            let white_pieces = self.position.pieces_by_color(Color::White);
            let black_pieces = self.position.pieces_by_color(Color::White);
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
