use crate::{Bitboard, Board, Color, Dir, Move, Piece, PieceKind, Ray, Square, ALL_DIRS, NOT_A_FILE, NOT_H_FILE};

impl Board {
    pub fn moves(&self) -> Vec<Move> {
        let color = self.color_to_move();
        let mut mvs = Vec::with_capacity(50);
        let queen = self[Piece::queen(color)];
        self.pawn_moves(&mut mvs, self[Piece::pawn(color)], color);
        self.knight_moves(&mut mvs, self[Piece::knight(color)], color);
        self.rook_moves(&mut mvs, self[Piece::rook(color)] | queen, color);
        self.bishop_moves(&mut mvs, self[Piece::bishop(color)] | queen, color);
        self.filter_moves_by_check(&mut mvs, color);
        self.king_moves(&mut mvs, color);
        mvs
    }

    pub fn moves_for_square(&self, square: Square) -> Vec<Move> {
        let mut mvs = Vec::new();
        let piece = self[square];
        let initial: Bitboard = square.into();
        if let Piece::Filled(kind, color) = piece {
            if color != self.color_to_move() {
                return mvs;
            }
            match kind {
                PieceKind::King => self.king_moves(&mut mvs, color),
                PieceKind::Queen => self.queen_moves(&mut mvs, initial, color),
                PieceKind::Bishop => self.bishop_moves(&mut mvs, initial, color),
                PieceKind::Rook => self.rook_moves(&mut mvs, initial, color),
                PieceKind::Knight => self.knight_moves(&mut mvs, initial, color),
                PieceKind::Pawn => self.pawn_moves(&mut mvs, initial, color),
            };
            self.filter_moves_by_check(&mut mvs, color);
        }
        mvs
    }

    pub fn filter_moves_by_check(&self, mvs: &mut Vec<Move>, color: Color) {
        let check_restriction = self.checkers;
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
            self[mv.origin].is_kind(PieceKind::King)
                || !(check_restriction & Bitboard::from(mv.dest)).is_empty()
                || (self[mv.origin] == Piece::pawn(color)
                    && Some(mv.dest) == self.ep_target
                    && Bitboard::from(ep_pawn) == check_restriction)
        });
    }

    fn king_moves(&self, mvs: &mut Vec<Move>, color: Color) {
        let attacks = self.attacks;
        let free = (self[Piece::Empty] | self[!color]) & !attacks;
        let initial = self[Piece::king(color)];

        for d in ALL_DIRS {
            moves(mvs, initial, Bitboard(0), free & d.filter(), d.offset());
        }

        let filter_offset = if color == Color::White { 56 } else { 0 };
        // Castling
        let ks_filter = Bitboard(0b00000110 << filter_offset);
        let ks_check = Bitboard(0b00001110 << filter_offset);
        let qs_filter = Bitboard(0b01110000 << filter_offset);
        let qs_check = Bitboard(0b00111000 << filter_offset);
        let castle_offset = if color == Color::White { 0 } else { 2 };
        if self.castle[castle_offset]
            && self[Piece::Empty] & ks_filter == ks_filter
            && (attacks & ks_check).is_empty()
        {
            moves(
                mvs,
                initial,
                Bitboard(0),
                self[Piece::Empty],
                Dir::East.offset() * 2,
            );
        }
        if self.castle[1 + castle_offset]
            && self[Piece::Empty] & qs_filter == qs_filter
            && (attacks & qs_check).is_empty()
        {
            moves(
                mvs,
                initial,
                Bitboard(0),
                self[Piece::Empty],
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
        let free = self[Piece::Empty];

        let pins = self.pins;
        let unpinned_pieces = !pins & initial;
        let pinned_pieces = pins & initial;
        // Single push
        moves_with_promotions(mvs, unpinned_pieces, Bitboard(0), free, color, dir.offset());

        //Double push
        let dp_free = free
            & if color == Color::White {
                (free >> 8) & Bitboard(0xff00000000u64)
            } else {
                (free << 8) & Bitboard(0xff000000u64)
            };
        moves(mvs, unpinned_pieces, Bitboard(0), dp_free, 2 * dir.offset());
        let pp = pinned_pieces;
        // Pinned double and single push
        for square in pp {
            let initial: Bitboard = square.into();
            let pin: Bitboard = self.pin_on_square(square).unwrap().into();
            moves_with_promotions(mvs, initial, Bitboard(0), free & pin, color, dir.offset());
            moves_with_promotions(
                mvs,
                initial,
                Bitboard(0),
                dp_free & pin,
                color,
                2 * dir.offset(),
            );
        }

        let ep_map: Bitboard = if let Some(sq) = self.ep_target {
            let king = self.king(color);
            let ep_pawn = if sq.rank() == 2 {
                Square(24 + sq.file())
            } else {
                Square(32 + sq.file())
            };
            if let Some(r) = Ray::from(king, ep_pawn) {
                let pieces: Vec<Piece> = r.into_iter().filter_map(|sqr| {
                    if self[sqr] == Piece::Empty {
                        None
                    } else {
                        Some(self[sqr])
                    }
                }).collect();
                if (r.dir == Dir::East || r.dir == Dir::West)
                    && pieces.len() >= 3
                    && ((pieces[0] == Piece::pawn(color) && pieces[1] == Piece::pawn(!color))
                        || (pieces[0] == Piece::pawn(!color) && pieces[1] == Piece::pawn(color)))
                    && (pieces[2] == Piece::rook(!color) || pieces[2] == Piece::queen(!color))
                {
                    Bitboard(0) // pinned
                } else {
                    sq.into()
                }
            } else {
                sq.into()
            }
        } else {
            Bitboard(0)
        };
        let cap = self[!color] | ep_map;

        let left_attack = if color == Color::White {
            Dir::NorWest
        } else {
            Dir::SouWest
        };
        moves_with_promotions(
            mvs,
            unpinned_pieces,
            Bitboard(0),
            cap & left_attack.filter(),
            color,
            left_attack.offset(),
        );
        for square in pinned_pieces {
            let bb: Bitboard = square.into();
            let pin: Bitboard = self.pin_on_square(square).unwrap().into();
            moves_with_promotions(
                mvs,
                bb,
                Bitboard(0),
                cap & pin & left_attack.filter(),
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
            Bitboard(0),
            cap & right_attack.filter(),
            color,
            right_attack.offset(),
        );
        for square in pinned_pieces {
            let pin: Bitboard = self.pin_on_square(square).unwrap().into();
            moves_with_promotions(
                mvs,
                square.into(),
                Bitboard(0),
                cap & pin & right_attack.filter(),
                color,
                right_attack.offset(),
            );
        }
    }
    fn knight_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [
            (
                Dir::North.offset() + Dir::NorEast.offset(),
                Dir::NorEast.filter(),
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
                Dir::SouEast.filter(),
            ),
            (
                Dir::South.offset() + Dir::SouWest.offset(),
                Dir::SouWest.filter(),
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
                Dir::NorWest.filter(),
            ),
        ];
        let cap = self[!color] | self[Piece::Empty];
        if cap.is_empty() {
            return;
        };

        let unpinned_pieces = initial & !self.pins;
        for (d, mask) in dirs {
            moves(mvs, unpinned_pieces, Bitboard(0), cap & mask, d);
        }
    }

    // Can't just & check, need to generate moves and then filter them...
    // Otherwise will prevent sliding moves from getting to where they can stop check
    fn rook_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [Dir::North, Dir::East, Dir::South, Dir::West];
        let cap = self[!color];
        let free = self[Piece::Empty];
        if free.is_empty() && cap.is_empty() {
            return;
        }

        let pins = self.pins;
        let unpinned_pieces = initial & !pins;
        let pinned_pieces: Bitboard = initial & pins;

        for d in dirs {
            moves(
                mvs,
                unpinned_pieces,
                free & d.filter(),
                cap & d.filter(),
                d.offset(),
            );
        }
        for square in pinned_pieces {
            let i: Bitboard = square.into();
            let pin = self.pin_on_square(square).unwrap();
            if pin.dir.piece_kind() == PieceKind::Rook {
                moves(
                    mvs,
                    i,
                    free & pin.dir.filter(),
                    cap & pin.dir.filter(),
                    pin.dir.offset(),
                );
                moves(
                    mvs,
                    i,
                    free & pin.dir.opposite().filter(),
                    cap & pin.dir.opposite().filter(),
                    pin.dir.opposite().offset(),
                );
            }
        }
    }
    fn bishop_moves(&self, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
        let dirs = [Dir::NorEast, Dir::SouEast, Dir::SouWest, Dir::NorWest];
        let cap = self[!color];
        let free = self[Piece::Empty];
        if free.is_empty() && cap.is_empty() {
            return;
        }

        let pins = self.pins;
        let unpinned_pieces = initial & !pins;
        let pinned_pieces = initial & pins;

        for d in dirs {
            moves(
                mvs,
                unpinned_pieces,
                free & d.filter(),
                cap & d.filter(),
                d.offset(),
            );
        }
        for square in pinned_pieces {
            let i: Bitboard = square.into();
            let pin = self.pin_on_square(square).unwrap();
            if dirs.contains(&pin.dir) {
                moves(
                    mvs,
                    i,
                    free & pin.dir.filter(),
                    cap & pin.dir.filter(),
                    pin.dir.offset(),
                );
                moves(
                    mvs,
                    i,
                    free & pin.dir.opposite().filter(),
                    cap & pin.dir.opposite().filter(),
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

#[inline(always)]
fn moves(mvs: &mut Vec<Move>, initial: Bitboard, free: Bitboard, cap: Bitboard, dir: i32) {
    let mut mv = initial << dir;
    let mut end = mv & free;
    let mut attacks = mv & cap;
    let mut offset_mult = 1;

    while !end.is_empty() || !attacks.is_empty() {
        for dest in end | attacks {
            let origin = Square((dest.index() as i32 - (dir * offset_mult)) as u8);
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            });
        }
        mv = end << dir;
        end = mv & free;
        attacks = mv & cap;
        offset_mult += 1;
    }
}

#[inline(always)]
fn moves_with_promotions(
    mvs: &mut Vec<Move>,
    initial: Bitboard,
    free: Bitboard,
    cap: Bitboard,
    color: Color,
    dir: i32,
) {
    let mut mv = initial << dir;
    let mut end = mv & free;
    let mut attacks = mv & cap;
    let mut offset_mult = 1;
    let promos = [
        Piece::queen(color),
        Piece::bishop(color),
        Piece::rook(color),
        Piece::knight(color),
    ];

    while !end.is_empty() || !attacks.is_empty() {
        for dest in end | attacks {
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
        mv = end << dir;
        end = mv & free;
        attacks = mv & cap;
        offset_mult += 1;
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
