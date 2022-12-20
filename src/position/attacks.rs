use crate::dir::{Dir, ALL_DIRS};
use crate::piece::Piece;
use crate::piece::{Color, PieceType};

use super::{Bitboard, Position};

pub const UP: i32 = -8;
pub const DOWN: i32 = 8;
pub const LEFT: i32 = 1;
pub const RIGHT: i32 = -1;
const NOT_H_FILE: u64 = 0xfefefefefefefefe;
const NOT_A_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
// Not sure if this will really help anything. hard to tell if a piece moving will leave the king
// in check. Goal is to generate legal moves at no more cost than pseudolegal moves.
// (well there will be a slight cost when moving pieces, but hopefully less than the cost to check
// every move to see if it leaves the king under attack)
impl Position {

    // Only for determining check
    pub(super) fn gen_attacks(&self, color: Color) -> Bitboard {
        let mut output = self.pawn_attacks(self[Piece::Filled(PieceType::Pawn, color)], color);
        output |= self.king_attacks(self[Piece::Filled(PieceType::King, color)]);
        output |= self.queen_moves(self[Piece::Filled(PieceType::Queen, color)], color);
        output |= self.bishop_moves(self[Piece::Filled(PieceType::Bishop, color)], color);
        output |= self.rook_moves(self[Piece::Filled(PieceType::Rook, color)], color);
        output |= self.knight_moves(self[Piece::Filled(PieceType::Knight, color)]);
        output
    }

    fn pawn_attacks(&self, initial: Bitboard, for_color: Color) -> Bitboard {
        let vertical = if for_color == Color::White {
            Dir::North.offset()
        } else {
            Dir::South.offset()
        };

        let mut output = moves(initial, 0, NOT_H_FILE, vertical + Dir::West.offset());
        output |= moves(initial, 0, NOT_A_FILE, vertical + Dir::East.offset());
        output
    }

    fn king_attacks(&self, initial: Bitboard) -> Bitboard {
        ALL_DIRS.into_iter().fold(0, |o, d| o | moves(initial, 0, d.mask(), d.offset()))
    }

    fn queen_moves(&self, initial: Bitboard, color: Color) -> Bitboard {
        self.bishop_moves(initial, color) | self.rook_moves(initial, color)
    }

    fn bishop_moves(&self, initial: Bitboard, color: Color) -> Bitboard {
        let f = self[Piece::Empty] | self[Piece::Filled(PieceType::King, !color)];
        let o = !f;

        let dirs = [
            Dir::NorEast,
            Dir::SouEast,
            Dir::SouWest,
            Dir::NorWest,
        ];

        let mut output = 0u64;
        for dir in dirs {
            output |= moves(initial, f & dir.mask(), o & dir.mask(), dir.offset());
        }
        output
    }

    fn rook_moves(&self, initial: Bitboard, color: Color) -> Bitboard {
        let f = self[Piece::Empty] | self[Piece::Filled(PieceType::King, !color)];
        let o = !f;

        let dirs = [
            Dir::North,
            Dir::East,
            Dir::South,
            Dir::West,
        ];

        let mut output = 0u64;
        for dir in dirs {
            output |= moves(initial, f & dir.mask(), o & dir.mask(), dir.offset());
        }
        output
    }

    fn knight_moves(&self, initial: Bitboard) -> Bitboard {
        let not_gh = 0xfcfcfcfcfcfcfcfc;
        let not_ab = 0x3f3f3f3f3f3f3f3f;

        let dirs = [
            (UP + UP + RIGHT, NOT_A_FILE),
            (UP + RIGHT + RIGHT, not_ab),
            (DOWN + RIGHT + RIGHT, not_ab),
            (DOWN + DOWN + RIGHT, NOT_A_FILE),
            (DOWN + DOWN + LEFT, NOT_H_FILE),
            (DOWN + LEFT + LEFT, not_gh),
            (UP + LEFT + LEFT, not_gh),
            (UP + UP + LEFT, NOT_H_FILE),
        ];

        let mut output = 0u64;
        for (dir, filter) in dirs {
            output |= moves(initial, 0, filter, dir);
        }
        output
    }

    pub(super) fn gen_checks(&self, color: Color) -> Vec<Bitboard> {
        let mut checks: Vec<Bitboard> = Vec::new();
        let king = Piece::Filled(PieceType::King, color);
        let initial = self[king];
        // Necessary for when assembling board and king isn't there yet
        if initial == 0 {
            return checks;
        }
        let free = self[Piece::Empty];
        let dirs = [
            Dir::North,
            Dir::NorEast,
            Dir::East,
            Dir::SouEast,
            Dir::South,
            Dir::SouWest,
            Dir::West,
            Dir::NorWest,
        ];
        // Doesn't check for pawns or knights
        for d in dirs {
            let cap = self[Piece::Filled(d.piece_type(), !color)]
                | self[Piece::Filled(PieceType::Queen, !color)];
            let mv = moves(initial, free & d.mask(), cap & d.mask(), d.offset());
            if mv & cap != 0 {
                checks.push(mv);
            }
        }
        let mut pawn_checks =
            self.pawn_attacks(initial, color) & self[Piece::Filled(PieceType::Pawn, !color)];
        while pawn_checks != 0 {
            let index = 63u8 - pawn_checks.leading_zeros() as u8;
            pawn_checks &= !(1 << index);
            checks.push(1 << index);
        }
        let mut knight_checks =
            self.knight_moves(initial) & self[Piece::Filled(PieceType::Knight, !color)];
        while knight_checks != 0 {
            let index = 63u8 - knight_checks.leading_zeros() as u8;
            knight_checks &= !(1 << index);
            checks.push(1 << index);
        }
        checks
    }

    pub(super) fn gen_pins(&self, color: Color) -> Vec<Bitboard> {
        let mut p: Vec<Bitboard> = Vec::new();
        let king = Piece::Filled(PieceType::King, color);
        let initial = self[king];
        let def = self.pieces_by_color(color) & !initial;
        // Necessary for when assembling board and king isn't there yet
        if initial == 0 {
            return p;
        }
        let free = self[Piece::Empty];
        let dirs = [
            Dir::North,
            Dir::NorEast,
            Dir::East,
            Dir::SouEast,
            Dir::South,
            Dir::SouWest,
            Dir::West,
            Dir::NorWest,
        ];
        // Need to handle when EP is pinned, but I don't keep track of ep target square or move
        // history here, both of which would be necessary to do it properly...
        for d in dirs {
            let filter = d.mask();
            let cap = self[Piece::Filled(d.piece_type(), !color)]
                | self[Piece::Filled(PieceType::Queen, !color)];
            let pin = pins(initial, free & filter, cap & filter, def & filter, d);
            if pin != 0 {
                p.push(pin);
            }
        }
        p
    }
}

fn moves(initial: Bitboard, free: Bitboard, cap: Bitboard, dir: i32) -> Bitboard {
    let mut output = 0;
    let shift = |x| {
        if dir.is_positive() {
            x << dir
        } else {
            x >> dir.abs()
        }
    };
    let mut mv = shift(initial);
    let mut end = mv & free;
    let mut attacks = mv & cap;

    while end > 0 || attacks > 0 {
        output |= end;
        output |= attacks;
        mv = shift(end);
        end = mv & free;
        attacks = mv & cap;
    }
    output
}

fn pins(initial: Bitboard, free: Bitboard, cap: Bitboard, def: Bitboard, dir: Dir) -> Bitboard {
    let mut output = 0u64;
    let mut pass = 1;
    let mut calc_end = |x: u64| {
        if x & def > 0 {
            pass -= 1;
        }
        x & (free | def)
    };
    let mut mv = dir.shift(initial);
    let mut end = calc_end(mv);
    let mut attacks = mv & cap;
    let mut hit = attacks != 0;

    while end > 0 || attacks > 0 {
        output |= end;
        output |= attacks;
        mv = dir.shift(end);
        end = calc_end(mv);
        attacks = mv & cap;
        if attacks != 0 {
            hit = true;
        }
    }
    if hit && pass == 0 {
        output
    } else {
        0
    }
}
