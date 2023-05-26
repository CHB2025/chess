use crate::{
    dir::{BISHOP_DIRS, ROOK_DIRS},
    Bitboard, Color, Dir, Piece, PieceKind, Ray, Square, ALL_DIRS, EMPTY, NOT_A_FILE, NOT_H_FILE,
};

pub(crate) fn moves(piece: Piece, square: Square, free: Bitboard) -> Bitboard {
    match piece {
        Piece::Empty => EMPTY,
        Piece::Filled(PieceKind::King, _) => king_moves(square),
        Piece::Filled(PieceKind::Pawn, color) => pawn_moves(color, square),
        Piece::Filled(PieceKind::Knight, _) => knight_moves(square),
        Piece::Filled(k, _) => sliding_moves(square, k, free),
    }
}

/// Ignores castling
fn king_moves(square: Square) -> Bitboard {
    ALL_DIRS
        .into_iter()
        .filter_map(|d| square.checked_add(d))
        .collect()
}

fn sliding_moves(square: Square, kind: PieceKind, free: Bitboard) -> Bitboard {
    let dirs: &[Dir] = match kind {
        PieceKind::Queen => &ALL_DIRS,
        PieceKind::Rook => &ROOK_DIRS,
        PieceKind::Bishop => &BISHOP_DIRS,
        _ => return EMPTY,
    };

    let mut result = EMPTY;

    for d in dirs {
        result |= Ray {
            origin: square,
            dir: *d,
        }
        .into_iter()
        .fold(
            EMPTY,
            |bb, sq| {
                if bb & free == bb {
                    bb | sq.into()
                } else {
                    bb
                }
            },
        )
    }
    result
}

fn knight_moves(square: Square) -> Bitboard {
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
    dirs.into_iter()
        .filter_map(|(offset, filter)| ((Bitboard::from(square) << offset) & filter).first_square())
        .collect()
}

// Should pushes even be in this? what value do they add? they are quick to add 
// in move_gen, but they get in the way for things like checks and 
fn pawn_moves(color: Color, square: Square/*, free: Bitboard*/) -> Bitboard {
    //let dir = if color == Color::White {
    //    Dir::North
    //} else {
    //    Dir::South
    //};

    //let dp_free = free
    //    & (free << dir)
    //    & match color {
    //        Color::White => Bitboard::new(0xff00000000u64),
    //        Color::Black => Bitboard::new(0xff000000u64),
    //    };
    let mut result = EMPTY;
    //if let Some(target) = square.checked_add(dir) {
    //    if free.contains(target) {
    //        result |= target.into();
    //    }
    //    if let Some(target) = target.checked_add(dir) {
    //        if dp_free.contains(target) {
    //            result |= target.into()
    //        }
    //    }
    //}

    let left_attack = match color {
        Color::White => Dir::NorWest,
        Color::Black => Dir::SouWest,
    };
    if let Some(target) = square.checked_add(left_attack) {
        result |= target.into();
    }

    let right_attack = match color {
        Color::White => Dir::NorEast,
        Color::Black => Dir::SouEast,
    };
    if let Some(target) = square.checked_add(right_attack) {
        result |= target.into();
    }
    result
}
