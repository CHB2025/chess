use crate::dir::{Dir, ALL_DIRS, NOT_A_FILE, NOT_H_FILE};
use crate::piece::{Color, PieceType};
use crate::ray::Ray;
use crate::{moves::Move, piece::Piece, position::Bitboard, square::Square, Board};

impl Board {
    pub fn moves(&self) -> Vec<Move> {
        let color = self.position.color_to_move();
        let mut mvs = Vec::new();
        self.pawn_moves(
            &mut mvs,
            self.position[Piece::Filled(PieceType::Pawn, color)],
            color,
        );
        self.knight_moves(
            &mut mvs,
            self.position[Piece::Filled(PieceType::Knight, color)],
            color,
        );
        self.rook_moves(
            &mut mvs,
            self.position[Piece::Filled(PieceType::Rook, color)]
                | self.position[Piece::Filled(PieceType::Queen, color)],
            color,
        );
        self.bishop_moves(
            &mut mvs,
            self.position[Piece::Filled(PieceType::Bishop, color)]
                | self.position[Piece::Filled(PieceType::Queen, color)],
            color,
        );
        self.filter_moves_by_check(&mut mvs, color);
        self.king_moves(&mut mvs, color);
        mvs
    }

    pub fn moves_for_square(&self, square: Square) -> Vec<Move> {
        let mut mvs = Vec::new();
        let piece = self.position[square];
        let initial = square.mask();
        if let Piece::Filled(kind, color) = piece {
            if color != self.position.color_to_move() {
                return mvs;
            }
            match kind {
                PieceType::King => self.king_moves(&mut mvs, color),
                PieceType::Queen => self.queen_moves(&mut mvs, initial, color),
                PieceType::Bishop => self.bishop_moves(&mut mvs, initial, color),
                PieceType::Rook => self.rook_moves(&mut mvs, initial, color),
                PieceType::Knight => self.knight_moves(&mut mvs, initial, color),
                PieceType::Pawn => self.pawn_moves(&mut mvs, initial, color),
            };
            self.filter_moves_by_check(&mut mvs, color);
        }
        mvs
    }

    pub fn filter_moves_by_check(&self, mvs: &mut Vec<Move>, color: Color) {
        let check_restriction = self.position.check_restrictions();
        let ep_pawn = if let Some(sq) = self.ep_target {
            if sq.rank() == 2 {
                Square(24 + sq.file())
            } else {
                Square(32 + sq.file())
            }
        } else {
            Square(64) // This is annoying
        };

        mvs.retain(|mv| {
            self.position[mv.origin].piece_type() == Some(PieceType::King)
                || check_restriction & mv.dest.mask() != 0
                || (self.position[mv.origin] == Piece::Filled(PieceType::Pawn, color)
                    && Some(mv.dest) == self.ep_target
                    && ep_pawn.mask() == check_restriction)
        });
    }

    fn king_moves(&self, mvs: &mut Vec<Move>, color: Color) {
        let attacks = self.position.attacks();
        let free = (self.position[Piece::Empty] | self.position.pieces_by_color(!color)) & !attacks;
        let initial = self.position[Piece::Filled(PieceType::King, color)];

        for d in ALL_DIRS {
            moves(mvs, initial, 0, free & d.mask(), d.offset());
        }

        let filter_offset = if color == Color::White { 56 } else { 0 };
        // Castling
        let ks_filter = 0b00000110 << filter_offset;
        let ks_check = 0b00001110 << filter_offset;
        let qs_filter = 0b01110000 << filter_offset;
        let qs_check = 0b00111000 << filter_offset;
        let castle_offset = if color == Color::White { 0 } else { 2 };
        if self.castle[castle_offset]
            && ks_filter & self.position[Piece::Empty] == ks_filter
            && ks_check & attacks == 0
        {
            moves(
                mvs,
                initial,
                0,
                self.position[Piece::Empty],
                Dir::East.offset() * 2,
            );
        }
        if self.castle[1 + castle_offset]
            && qs_filter & self.position[Piece::Empty] == qs_filter
            && qs_check & attacks == 0
        {
            moves(
                mvs,
                initial,
                0,
                self.position[Piece::Empty],
                Dir::West.offset() * 2,
            );
        }
    }

    fn pawn_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dir = if color == Color::White {
            Dir::North
        } else {
            Dir::South
        };
        let free = self.position[Piece::Empty];

        let pins = self.position.pins();
        let unpinned_pieces = !pins & initial;
        let pinned_pieces = pins & initial;
        // Single push
        moves_with_promotions(mvs, unpinned_pieces, 0, free, color, dir.offset());

        //Double push
        let dp_free = free & if color == Color::White {
            (free >> 8) & 0xff00000000
        } else {
            (free << 8) & 0xff000000
        };
        moves(mvs, unpinned_pieces, 0, dp_free, 2 * dir.offset());
        let mut pp = pinned_pieces;
        // Pinned double and single push
        while pp != 0 {
            let square = Square(63 - pp.leading_zeros() as u8);
            pp &= !square.mask();
            let pin = self.position.pin_on_square(square).unwrap();
            moves_with_promotions(
                mvs,
                square.mask(),
                0,
                free & pin.mask(),
                color,
                dir.offset(),
            );
            moves_with_promotions(
                mvs,
                square.mask(),
                0,
                dp_free & pin.mask(),
                color,
                2 * dir.offset(),
            );
        }

        let ep_map = if let Some(sq) = self.ep_target {
            let king = Square(
                63 - self.position[Piece::Filled(PieceType::King, color)].leading_zeros() as u8,
            );
            let ep_pawn = if sq.rank() == 2 {
                Square(24 + sq.file())
            } else {
                Square(32 + sq.file())
            };
            if let Some(r) = Ray::from(king, ep_pawn) {
                let pieces = r.pieces(&self.position);
                if (r.dir == Dir::East || r.dir == Dir::West)
                    && pieces.len() >= 3
                    && ((pieces[0] == Piece::Filled(PieceType::Pawn, color)
                        && pieces[1] == Piece::Filled(PieceType::Pawn, !color))
                        || (pieces[0] == Piece::Filled(PieceType::Pawn, !color)
                            && pieces[1] == Piece::Filled(PieceType::Pawn, color)))
                    && (pieces[2] == Piece::Filled(PieceType::Rook, !color)
                        || pieces[2] == Piece::Filled(PieceType::Queen, !color))
                {
                    0 // pinned
                } else {
                    sq.mask()
                }
            } else {
                sq.mask()
            }
        } else {
            0
        };
        let cap = self.position.pieces_by_color(!color) | ep_map;

        let left_attack = if color == Color::White {
            Dir::NorWest
        } else {
            Dir::SouWest
        };
        moves_with_promotions(
            mvs,
            unpinned_pieces,
            0,
            cap & left_attack.mask(),
            color,
            left_attack.offset(),
        );
        let mut pp = pinned_pieces;
        while pp != 0 {
            let square = Square(63 - pp.leading_zeros() as u8);
            pp &= !square.mask();
            let pin = self.position.pin_on_square(square).unwrap();
            moves_with_promotions(
                mvs,
                square.mask(),
                0,
                cap & pin.mask() & left_attack.mask(),
                color,
                left_attack.offset(),
            );
        }

        let right_attack = if color == Color::White {
            Dir::NorEast
        } else {
            Dir::SouEast
        };
        moves_with_promotions(
            mvs,
            unpinned_pieces,
            0,
            cap & right_attack.mask(),
            color,
            right_attack.offset(),
        );
        let mut pp = pinned_pieces;
        while pp != 0 {
            let square = Square(63 - pp.leading_zeros() as u8);
            pp &= !square.mask();
            let pin = self.position.pin_on_square(square).unwrap();
            moves_with_promotions(
                mvs,
                square.mask(),
                0,
                cap & pin.mask() & right_attack.mask(),
                color,
                right_attack.offset(),
            );
        }
    }
    fn knight_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [
            (
                Dir::North.offset() + Dir::NorEast.offset(),
                Dir::NorEast.mask(),
            ),
            (
                Dir::NorEast.offset() + Dir::East.offset(),
                NOT_A_FILE & (NOT_A_FILE >> 1),
            ),
            (
                Dir::SouEast.offset() + Dir::East.offset(),
                NOT_A_FILE & (NOT_A_FILE >> 1),
            ),
            (
                Dir::South.offset() + Dir::SouEast.offset(),
                Dir::SouEast.mask(),
            ),
            (
                Dir::South.offset() + Dir::SouWest.offset(),
                Dir::SouWest.mask(),
            ),
            (
                Dir::SouWest.offset() + Dir::West.offset(),
                NOT_H_FILE & (NOT_H_FILE << 1),
            ),
            (
                Dir::NorWest.offset() + Dir::West.offset(),
                NOT_H_FILE & (NOT_H_FILE << 1),
            ),
            (
                Dir::North.offset() + Dir::NorWest.offset(),
                Dir::NorWest.mask(),
            ),
        ];
        let cap = self.position.pieces_by_color(!color) | self.position[Piece::Empty];
        if cap == 0 {
            return;
        };

        let unpinned_pieces = initial & !self.position.pins();
        for (d, mask) in dirs {
            moves(mvs, unpinned_pieces, 0, cap & mask, d);
        }
    }

    // Can't just & check, need to generate moves and then filter them...
    // Otherwise will prevent sliding moves from getting to where they can stop check
    fn rook_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [Dir::North, Dir::East, Dir::South, Dir::West];
        let cap = self.position.pieces_by_color(!color);
        let free = self.position[Piece::Empty];
        if free == 0 && cap == 0 {
            return;
        }

        let pins = self.position.pins();
        let unpinned_pieces = initial & !pins;
        let mut pinned_pieces: Bitboard = initial & pins;

        for d in dirs {
            moves(
                mvs,
                unpinned_pieces,
                free & d.mask(),
                cap & d.mask(),
                d.offset(),
            );
        }
        while pinned_pieces != 0 {
            let square = Square(63 - pinned_pieces.leading_zeros() as u8);
            let i = square.mask();
            pinned_pieces &= !i;
            let pin = self.position.pin_on_square(square).unwrap();
            if dirs.contains(&pin.dir) {
                moves(
                    mvs,
                    i,
                    free & pin.dir.mask(),
                    cap & pin.dir.mask(),
                    pin.dir.offset(),
                );
                moves(
                    mvs,
                    i,
                    free & pin.dir.opposite().mask(),
                    cap & pin.dir.opposite().mask(),
                    pin.dir.opposite().offset(),
                );
            }
        }
    }
    fn bishop_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [Dir::NorEast, Dir::SouEast, Dir::SouWest, Dir::NorWest];
        let cap = self.position.pieces_by_color(!color);
        let free = self.position[Piece::Empty];
        if free == 0 && cap == 0 {
            return;
        }

        let pins = self.position.pins();
        let unpinned_pieces = initial & !pins;
        let mut pinned_pieces = initial & pins;

        for d in dirs {
            moves(
                mvs,
                unpinned_pieces,
                free & d.mask(),
                cap & d.mask(),
                d.offset(),
            );
        }
        while pinned_pieces != 0 {
            let square = Square(63 - pinned_pieces.leading_zeros() as u8);
            let i = square.mask();
            pinned_pieces &= !i;
            let pin = self.position.pin_on_square(square).unwrap();
            if dirs.contains(&pin.dir) {
                moves(
                    mvs,
                    i,
                    free & pin.dir.mask(),
                    cap & pin.dir.mask(),
                    pin.dir.offset(),
                );
                moves(
                    mvs,
                    i,
                    free & pin.dir.opposite().mask(),
                    cap & pin.dir.opposite().mask(),
                    pin.dir.opposite().offset(),
                )
            }
        }
    }
    fn queen_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        self.rook_moves(mvs, initial, color);
        self.bishop_moves(mvs, initial, color);
    }
}

fn moves(mvs: &mut Vec<Move>, initial: Bitboard, free: Bitboard, cap: Bitboard, dir: i32) {
    let mut mv = shift(initial, dir);
    let mut end = mv & free;
    let mut attacks = mv & cap;
    let mut offset_mult = 1;

    while end > 0 || attacks > 0 {
        let mut targets = end | attacks;
        while targets.leading_zeros() != 64 {
            let dest = Square(63u8 - targets.leading_zeros() as u8);
            targets &= !dest.mask();
            let origin = Square((dest.index() as i32 - (dir * offset_mult)) as u8);
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            });
        }
        mv = shift(end, dir);
        end = mv & free;
        attacks = mv & cap;
        offset_mult += 1;
    }
}

fn moves_with_promotions(
    mvs: &mut Vec<Move>,
    initial: Bitboard,
    free: Bitboard,
    cap: Bitboard,
    color: Color,
    dir: i32,
) {
    let mut mv = shift(initial, dir);
    let mut end = mv & free;
    let mut attacks = mv & cap;
    let mut offset_mult = 1;
    let promos = [
        Piece::Filled(PieceType::Queen, color),
        Piece::Filled(PieceType::Bishop, color),
        Piece::Filled(PieceType::Rook, color),
        Piece::Filled(PieceType::Knight, color),
    ];

    while end > 0 || attacks > 0 {
        let mut targets = end | attacks;
        while targets.leading_zeros() != 64 {
            let dest = Square(63u8 - targets.leading_zeros() as u8);
            targets &= !dest.mask();
            let origin = Square((dest.index() as i32 - (dir * offset_mult)) as u8);
            if dest.rank() == 0 || dest.rank() == 7 {
                for promotion in promos {
                    mvs.push(Move {
                        origin,
                        dest,
                        promotion,
                    });
                }
            } else {
                mvs.push(Move {
                    origin,
                    dest,
                    promotion: Piece::Empty,
                });
            }
        }
        mv = shift(end, dir);
        end = mv & free;
        attacks = mv & cap;
        offset_mult += 1;
    }
}

fn shift(initial: Bitboard, dir: i32) -> Bitboard {
    if dir.is_positive() {
        initial << dir.abs()
    } else {
        initial >> dir.abs()
    }
}

#[cfg(test)]
mod tests {
    use crate::moves::Move;
    use crate::Board;

    const POSITIONS: [(&str, usize); 6] = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            20,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            48,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 14),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            6,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            44,
        ),
        (
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            46,
        ),
    ];

    #[test]
    fn test_moves() {
        for (fen, moves) in POSITIONS {
            let game = Board::from_fen(fen).unwrap();
            let m = game.moves();
            if moves != m.len() {
                println!(
                    "Generated moves from game with fen \"{}\" does not match expectations.",
                    fen
                );
                println!(
                    "Moves generated:\n{}",
                    m.iter().fold(String::new(), |sum: String, m: &Move| sum
                        + &format!("{}\n", m))
                );
            }
            assert_eq!(moves, m.len());
        }
    }
}
