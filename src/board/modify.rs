use super::*;

pub struct Modifier<'a> {
    pub(super) board: &'a mut Board,
}

impl<'a> Modifier<'a> {
    /// Puts the given piece at the provided square. Returns the piece that was replaced
    #[inline(always)]
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.board.pieces[square];
        self.board.pieces[square] = piece;

        let map: Bitboard = square.into();
        self.board.bitboards[replaced] ^= map;
        if let Some(color) = replaced.color() {
            self.board.color_bitboards[color] ^= map;
            hash::increment_hash(self.board, replaced, square)
        }
        self.board.bitboards[piece] |= map;
        if let Some(color) = piece.color() {
            self.board.color_bitboards[color] |= map;
            hash::increment_hash(self.board, piece, square);
        }

        replaced
    }

    /// Clears the provided square. Returns the piece that previously held that position
    #[inline(always)]
    pub fn clear(&mut self, square: Square) -> Piece {
        self.put(Piece::Empty, square)
    }

    /// Moves the piece at `from` to `to`, replacing it with Piece::Empty.
    /// Returns the piece that was replaced at `to`.
    #[inline(always)]
    pub fn r#move(&mut self, from: Square, to: Square) -> Piece {
        self.move_replace(from, to, Piece::Empty)
    }

    #[inline(always)]
    pub fn move_replace(&mut self, from: Square, to: Square, replacement: Piece) -> Piece {
        let piece = self.put(replacement, from);
        self.put(piece, to)
    }

    #[inline(always)]
    pub fn toggle_color_to_move(&mut self) -> Color {
        self.board.color_to_move = !self.board.color_to_move;
        hash::toggle_color_hash(self.board);
        self.board.color_to_move
    }

    #[inline(always)]
    pub fn set_castle(&mut self, color: Color, castle: Castle) {
        hash::update_castle_hash(self.board, color, self.board.castle[color], castle);
        self.board.castle[color] = castle;
    }

    #[inline(always)]
    pub fn set_ep_target(&mut self, target: Option<Square>) {
        if let Some(sqr) = self.board.ep_target {
            hash::toggle_ep_hash(self.board, sqr);
        }
        self.board.ep_target = target;
        if let Some(sqr) = target {
            hash::toggle_ep_hash(self.board, sqr);
        }
    }

    #[inline(always)]
    pub fn board(&self) -> &Board {
        self.board
    }

    #[inline(always)]
    pub(super) fn complete(&mut self) {
        self.board.update_position();
    }
}
