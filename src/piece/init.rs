use super::*;

impl Piece {
    pub fn king(color: Color) -> Piece {
        Piece::Filled(PieceKind::King, color)
    }

    pub fn queen(color: Color) -> Self {
        Piece::Filled(PieceKind::Queen, color)
    }

    pub fn bishop(color: Color) -> Self {
        Piece::Filled(PieceKind::Bishop, color)
    }

    pub fn rook(color: Color) -> Self {
        Piece::Filled(PieceKind::Rook, color)
    }

    pub fn knight(color: Color) -> Self {
        Piece::Filled(PieceKind::Knight, color)
    }

    pub fn pawn(color: Color) -> Self {
        Piece::Filled(PieceKind::Pawn, color)
    }
}
