use crate::dir::{Dir, ALL_DIRS};
use crate::piece::Piece;
use crate::piece::Color;

use super::{Bitboard, Position};

const UP: i32 = -8;
const DOWN: i32 = 8;
const LEFT: i32 = 1;
const RIGHT: i32 = -1;
const NOT_H_FILE: u64 = 0xfefefefefefefefe;
const NOT_A_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
impl Position {
    // Only for determining check
    pub(super) fn gen_attacks(&self, color: Color) -> Bitboard {
        let queen = self[Piece::queen(color)];
        let mut output = self.pawn_attacks(self[Piece::pawn(color)], color);
        output |= self.king_attacks(self[Piece::king(color)]);
        output |= self.bishop_moves(self[Piece::bishop(color)] | queen, color);
        output |= self.rook_moves(self[Piece::rook(color)] | queen, color);
        output | self.knight_moves(self[Piece::knight(color)])
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
        ALL_DIRS
            .into_iter()
            .fold(0, |o, d| o | moves(initial, 0, d.mask(), d.offset()))
    }

    fn bishop_moves(&self, initial: Bitboard, color: Color) -> Bitboard {
        let f = self[Piece::Empty] | self[Piece::king(!color)];
        let o = !f;

        let dirs = [Dir::NorEast, Dir::SouEast, Dir::SouWest, Dir::NorWest];

        let mut output = 0u64;
        for dir in dirs {
            output |= moves(initial, f & dir.mask(), o & dir.mask(), dir.offset());
        }
        output
    }

    fn rook_moves(&self, initial: Bitboard, color: Color) -> Bitboard {
        let f = self[Piece::Empty] | self[Piece::king(!color)];
        let o = !f;

        let dirs = [Dir::North, Dir::East, Dir::South, Dir::West];

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

    pub(super) fn update_pins_and_checks(&mut self) {
        let color = self.color_to_move;
        if !self.king_exists(color) {
            self.pins = 0;
            self.checks = !0;
            return;
        }
        let mut p: Bitboard = 0;
        let mut c: Bitboard = !0;
        let initial = self.king(color).mask();
        let def = self[color] & !initial;
        let free = self[Piece::Empty];
        let queen = self[Piece::queen(!color)];

        for d in ALL_DIRS {
            let filter = d.mask();
            let cap = self[Piece::Filled(d.piece_kind(), !color)] | queen;
            let (bitboard, is_pin) = pins(initial, free & filter, cap & filter, def & filter, d);
            if is_pin {
                p |= bitboard;
            } else {
                c &= bitboard;
            }
        }

        let pawn_attacks =
            self.pawn_attacks(initial, color) & self[Piece::pawn(!color)];
        if pawn_attacks != 0 {
            c &= pawn_attacks;
        }

        let knight_attacks =
            self.knight_moves(initial) & self[Piece::knight(!color)];
        if knight_attacks != 0 {
            c &= knight_attacks;
        }

        self.pins = p;
        self.checks = c;
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

    while end != 0 || attacks != 0{
        output |= end;
        output |= attacks;
        mv = shift(end);
        end = mv & free;
        attacks = mv & cap;
    }
    output
}

fn pins(
    initial: Bitboard,
    free: Bitboard,
    cap: Bitboard,
    def: Bitboard,
    dir: Dir,
) -> (Bitboard, bool) {
    let mut output = 0u64;
    let mut is_pin = false;
    let mut mv = dir.shift(initial);
    let mut end = mv & free;
    let mut pin = mv & def;

    while end != 0 || (pin != 0 && !is_pin) {
        output |= end;
        if pin != 0 {
            output |= pin;
            is_pin = true;
        }
        mv = dir.shift(end | pin);
        end = mv & free;
        pin = mv & def;
    }

    let attacks = mv & cap;
    if attacks != 0 {
        output |= attacks;
        (output, is_pin)
    } else {
        (!0, false)
    }
}
