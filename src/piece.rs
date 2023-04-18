#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt, ops, str};

use crate::error::{BoardError, ErrorKind};

pub const PROMO_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Bishop,
    PieceKind::Knight,
    PieceKind::Rook,
];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Piece {
    Filled(PieceKind, Color),
    Empty,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PieceKind {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
            _ => unreachable!(),
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

impl str::FromStr for Color {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Attempted to parse color from invalid string",
            )),
        }
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
    pub fn is_color(&self, color: Color) -> bool {
        match self {
            Self::Filled(_, c) => *c == color,
            Self::Empty => false,
        }
    }
    pub fn kind(&self) -> Option<PieceKind> {
        match self {
            Self::Filled(t, _) => Some(*t),
            Self::Empty => None,
        }
    }
    pub fn is_kind(&self, kind: PieceKind) -> bool {
        match self {
            Self::Filled(k, _) => *k == kind,
            Self::Empty => false,
        }
    }

    // Easy Piece creation
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
