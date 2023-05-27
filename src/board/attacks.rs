use super::Board;
use crate::{Bitboard, Check, Color, Dir, Piece, ALL_DIRS, EMPTY, NOT_A_FILE, NOT_H_FILE};

const UP: i32 = -8;
const DOWN: i32 = 8;
const LEFT: i32 = 1;
const RIGHT: i32 = -1;

impl Board {
    #[inline]
    pub(super) fn update_position(&mut self) {
        self.update_pins_and_checks();
        //self.attacks = self.gen_attacks(!self.color_to_move);
    }

    #[inline]
    fn pawn_attacks(&self, initial: Bitboard, for_color: Color) -> Bitboard {
        let vertical = if for_color == Color::White {
            Dir::North.offset()
        } else {
            Dir::South.offset()
        };

        let mut output = moves(initial, EMPTY, NOT_H_FILE, vertical + Dir::West.offset());
        output |= moves(initial, EMPTY, NOT_A_FILE, vertical + Dir::East.offset());
        output
    }

    #[inline]
    fn knight_attacks(&self, initial: Bitboard) -> Bitboard {
        let not_gh = Bitboard::new(0xfcfcfcfcfcfcfcfc);
        let not_ab = Bitboard::new(0x3f3f3f3f3f3f3f3f);

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

        let mut output = EMPTY;
        for (dir, filter) in dirs {
            output |= moves(initial, EMPTY, filter, dir);
        }
        output
    }

    #[inline]
    pub(super) fn update_pins_and_checks(&mut self) {
        let mut p = EMPTY;
        let mut c = EMPTY;
        let color = self.color_to_move;

        let initial: Bitboard = self.king(color).into();
        let def = self[color] ^ initial;
        let free = self[Piece::Empty];
        let queen = self[Piece::queen(!color)];

        for d in ALL_DIRS {
            let filter = d.filter();
            let cap = self[Piece::Filled(d.piece_kind(), !color)] | queen;
            let (bitboard, is_pin) = pins(initial, free & filter, cap & filter, def & filter, d);
            if is_pin {
                p |= bitboard;
            } else {
                c |= bitboard;
            }
        }

        if c.count_squares() < 2 {
            let pawn_attacks = self.pawn_attacks(initial, color) & self[Piece::pawn(!color)];
            if !pawn_attacks.is_empty() {
                c |= pawn_attacks;
            }
        }

        let knight_attacks = self.knight_attacks(initial) & self[Piece::knight(!color)];
        if !knight_attacks.is_empty() {
            c |= knight_attacks;
        }

        self.pins = p;
        self.check = match c.count_squares() {
            0 => Check::None,
            1 => Check::Single(c.first_square().expect("match says there's a square")),
            _ => Check::Double(c),
        }
    }
}

#[inline]
fn moves(initial: Bitboard, free: Bitboard, cap: Bitboard, dir: i32) -> Bitboard {
    let mut output = EMPTY;
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

    while !end.is_empty() || !attacks.is_empty() {
        output |= end;
        output |= attacks;
        mv = shift(end);
        end = mv & free;
        attacks = mv & cap;
    }
    output
}

#[inline]
fn pins(
    initial: Bitboard,
    free: Bitboard,
    cap: Bitboard,
    def: Bitboard,
    dir: Dir,
) -> (Bitboard, bool) {
    let mut output = EMPTY;
    let mut is_pin = false;
    let mut mv = initial << dir;
    let mut end = mv & free;
    let mut pin = mv & def;

    while !end.is_empty() || (!pin.is_empty() && !is_pin) {
        if !pin.is_empty() {
            output |= pin;
            is_pin = true;
        }
        mv = (end | pin) << dir;
        end = mv & free;
        pin = mv & def;
    }

    let attacks = mv & cap;
    if !attacks.is_empty() {
        output |= attacks;
        (output, is_pin)
    } else {
        (EMPTY, false)
    }
}
