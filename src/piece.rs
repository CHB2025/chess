use std::{fmt, str, ops};

use crate::error::{BoardError, ErrorKind};

pub mod init;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Piece {
    Filled(PieceKind, Color),
    Empty,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum PieceKind {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Color {
    White,
    Black,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Filled(p, c) => {
                let mut ch = format!("{}", p);
                if *c == Color::White {
                    ch = ch.to_uppercase();
                }
                write!(f, "{ch}")
            }
            Self::Empty => write!(f, "-"),
        }
    }
}

impl str::FromStr for Piece {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Piece::try_from(s.parse::<char>()?)
    }
}

impl TryFrom<char> for Piece {
    type Error = BoardError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let color = if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Ok(match c.to_ascii_lowercase() {
            'k' => Piece::king(color),
            'q' => Piece::queen(color),
            'b' => Piece::bishop(color),
            'n' => Piece::knight(color),
            'r' => Piece::rook(color),
            'p' => Piece::pawn(color),
            '-' => Piece::Empty,
            _ => {
                return Err(BoardError::new(
                    ErrorKind::InvalidInput,
                    "Illegal character for piece",
                ))
            }
        })
    }
}

impl TryFrom<usize> for Piece {
    type Error = BoardError;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index > 12 {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of bounds for piece",
            ));
        }
        let color = if (index & 1) == 1 {
            Color::Black
        } else {
            Color::White
        };
        Ok(match index >> 1 {
            0 => Self::Filled(PieceKind::King, color),
            1 => Self::Filled(PieceKind::Queen, color),
            2 => Self::Filled(PieceKind::Bishop, color),
            3 => Self::Filled(PieceKind::Knight, color),
            4 => Self::Filled(PieceKind::Rook, color),
            5 => Self::Filled(PieceKind::Pawn, color),
            6 => Self::Empty,
            _ => unreachable!()
        })
    }
}

impl fmt::Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::King => "k",
            Self::Queen => "q",
            Self::Bishop => "b",
            Self::Knight => "n",
            Self::Rook => "r",
            Self::Pawn => "p",
        };
        write!(f, "{c}")
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::White => "w",
            Self::Black => "b",
        };
        write!(f, "{c}")
    }
}

impl ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Piece {
    pub fn index(&self) -> usize {
        match self {
            Self::Filled(p, c) => (*p as usize) << 1 | (*c as usize),
            Self::Empty => 12,
        }
    }
    pub fn color(&self) -> Option<Color> {
        match self {
            Self::Filled(_, c) => Some(*c),
            Self::Empty => None,
        }
    }
    pub fn kind(&self) -> Option<PieceKind> {
        match self {
            Self::Filled(t, _) => Some(*t),
            Self::Empty => None,
        }
    }
}
