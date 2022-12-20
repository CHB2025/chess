use std::{fmt, str, ops};

use crate::error::{BoardError, ErrorKind};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Piece {
    Filled(PieceType, Color),
    Empty,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum PieceType {
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
            'k' => Piece::Filled(PieceType::King, color),
            'q' => Piece::Filled(PieceType::Queen, color),
            'b' => Piece::Filled(PieceType::Bishop, color),
            'n' => Piece::Filled(PieceType::Knight, color),
            'r' => Piece::Filled(PieceType::Rook, color),
            'p' => Piece::Filled(PieceType::Pawn, color),
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
            0 => Self::Filled(PieceType::King, color),
            1 => Self::Filled(PieceType::Queen, color),
            2 => Self::Filled(PieceType::Bishop, color),
            3 => Self::Filled(PieceType::Knight, color),
            4 => Self::Filled(PieceType::Rook, color),
            5 => Self::Filled(PieceType::Pawn, color),
            6 => Self::Empty,
            _ => unreachable!()
        })
    }
}

impl fmt::Display for PieceType {
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
    pub fn piece_type(&self) -> Option<PieceType> {
        match self {
            Self::Filled(t, _) => Some(*t),
            Self::Empty => None,
        }
    }
}
