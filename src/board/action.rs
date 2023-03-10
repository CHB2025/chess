use super::*;

pub struct Action<'a> {
    pub(super) board: &'a mut Board,
}

impl<'a> Action<'a> {
    /// Puts the given piece at the provided square. Returns the piece that was replaced
    #[inline(always)]
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.board.pieces[square];
        self.board.pieces[square] = piece;

        let map:Bitboard = square.into();
        self.board.bitboards[replaced] ^= map;
        if let Some(color) = replaced.color() {
            self.board.color_bitboards[color] ^= map;
            self.board.increment_hash(replaced, square)
        }
        self.board.bitboards[piece] |= map;
        if let Some(color) = piece.color() {
            self.board.color_bitboards[color] |= map;
            self.board.increment_hash(piece, square);
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
    pub fn board(&self) -> &Board {
        self.board
    }

    #[inline(always)]
    pub(super) fn complete(&mut self) {
        self.board.color_to_move = !self.board.color_to_move;
        self.board.update_position();
    }
}
