use super::Board;
use crate::{Bitboard, Check, Piece, PieceKind, Ray, Square, ALL_DIRS};

impl Board {
    #[inline]
    pub(super) fn update_position(&mut self) {
        let color = self.color_to_move;

        let king = self.king(color);

        self.pins = ALL_DIRS
            .into_iter()
            .filter_map(|d| {
                let r = Ray {
                    origin: king,
                    dir: d,
                };
                let mut pin: Option<Square> = None;
                for sq in r {
                    match (self[sq], pin) {
                        (Piece::Empty, _) => continue,
                        (Piece::Filled(_, c), None) if c == color => pin = Some(sq),
                        (Piece::Filled(kind, c), p)
                            if (kind == d.piece_kind() || kind == PieceKind::Queen)
                                && c == !color =>
                        {
                            return p;
                        }
                        _ => return None,
                    };
                }
                None
            })
            .collect();

        let c: Bitboard = self.color_bitboards[!color]
            .into_iter()
            .filter(|sq| self.move_cache[*sq].contains(king))
            .collect();
        self.check = match c.count_squares() {
            0 => Check::None,
            1 => Check::Single(c.first_square().expect("match says there's a square")),
            _ => Check::Double(c),
        }
    }
}
