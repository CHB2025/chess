use crate::{PieceKind, Bitboard, ALL, NOT_A_FILE, NOT_H_FILE};

pub const ALL_DIRS: [Dir; 8] = [
    Dir::North,
    Dir::NorEast,
    Dir::East,
    Dir::SouEast,
    Dir::South,
    Dir::SouWest,
    Dir::West,
    Dir::NorWest,
];
pub const ROOK_DIRS: [Dir; 4] = [
    Dir::North,
    Dir::East,
    Dir::South,
    Dir::West,
];
pub const BISHOP_DIRS: [Dir; 4] = [
    Dir::NorEast,
    Dir::SouEast,
    Dir::SouWest,
    Dir::NorWest,
];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Dir {
    North,
    NorEast,
    East,
    SouEast,
    South,
    SouWest,
    West,
    NorWest,
}

impl Dir {
    pub const fn offset(&self) -> i32 {
        match self {
            Dir::North => -8,
            Dir::NorEast => -9,
            Dir::East => -1,
            Dir::SouEast => 7,
            Dir::South => 8,
            Dir::SouWest => 9,
            Dir::West => 1,
            Dir::NorWest => -7,
        }
    }

    pub const fn opposite(&self) -> Dir {
        match self {
            Dir::North => Dir::South,
            Dir::NorEast => Dir::SouWest,
            Dir::East => Dir::West, 
            Dir::SouEast => Dir::NorWest,
            Dir::South => Dir::North,
            Dir::SouWest => Dir::NorEast,
            Dir::West => Dir::East,
            Dir::NorWest => Dir::SouEast,
        }
    }

    pub const fn filter(&self) -> Bitboard {
        match self {
            Dir::North => ALL,
            Dir::NorEast => NOT_A_FILE,
            Dir::East => NOT_A_FILE,
            Dir::SouEast => NOT_A_FILE,
            Dir::South => ALL,
            Dir::SouWest => NOT_H_FILE,
            Dir::West => NOT_H_FILE,
            Dir::NorWest => NOT_H_FILE,
        }
        //
    }

    /// Returns the piece (either rook or bishop) that moves in this direction. Ignores Queen,
    /// which moves in all directions, and King, Knight, and Pawn, which have unique movement.
    pub const fn piece_kind(&self) -> PieceKind {
        if self.offset().abs() == 8 || self.offset().abs() == 1 {
            PieceKind::Rook
        } else {
            PieceKind::Bishop
        }
    }

    pub const fn horizontal_offset(&self) -> i32 {
        match self {
            Dir::North | Dir::South => 0,
            Dir::NorEast | Dir::SouEast | Dir::East => -1,
            Dir::NorWest | Dir::SouWest | Dir::West => 1,
        }
    }
}
