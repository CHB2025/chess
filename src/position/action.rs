use super::*;

pub struct Action<'a> {
    pub(super) position: &'a mut Position,
}

impl<'a> Action<'a> {
    /// Puts the given piece at the provided square. Returns the piece that was replaced
    pub fn put(&mut self, piece: Piece, square: Square) -> Piece {
        let replaced = self.position.pieces[square];
        self.position.pieces[square] = piece;

        let map = square.mask();
        self.position.bitboards[replaced] &= !map;
        if let Some(color) = replaced.color() {
            self.position.colors[color] &= !map;
        }
        self.position.bitboards[piece] |= map;
        if let Some(color) = piece.color() {
            self.position.colors[color] |= map;
        }

        replaced
    }

    /// Clears the provided square. Returns the piece that previously held that position
    pub fn clear(&mut self, square: Square) -> Piece {
        self.put(Piece::Empty, square)
    }

    /// Moves the piece at `from` to `to`, replacing it with Piece::Empty.
    /// Returns the piece that was replaced at `to`.
    pub fn r#move(&mut self, from: Square, to: Square) -> Piece {
        self.move_replace(from, to, Piece::Empty)
    }

    pub fn move_replace(&mut self, from: Square, to: Square, replacement: Piece) -> Piece {
        let piece = self.put(replacement, from);
        self.put(piece, to)
    }

    pub fn position(&'a self) -> &'a Position {
        self.position
    }

    pub(super) fn complete(&mut self) {
        self.position.color_to_move = !self.position.color_to_move;
        self.position.update_attacks_and_pins();
    }
}
