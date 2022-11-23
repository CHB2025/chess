use std::{fmt, str};

use crate::error::{BoardError, ErrorKind};

const TYPE_MASK: u8 = 7;
pub const BLACK: usize = 8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Piece {
    King(bool),
    Queen(bool),
    Bishop(bool),
    Knight(bool),
    Rook(bool),
    Pawn(bool),
    Empty,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut c = match self {
            Piece::King(_) => 'k',
            Piece::Queen(_) => 'q',
            Piece::Bishop(_) => 'b',
            Piece::Knight(_) => 'n',
            Piece::Rook(_) => 'r',
            Piece::Pawn(_) => 'p',
            Piece::Empty => '-',
        };

        if self.is_white() {
            c = c.to_ascii_uppercase();
        }
        write!(f, "{c}")
    }
}

impl str::FromStr for Piece {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c: char = s.parse().map_err(|_| {
            BoardError::new(ErrorKind::InvalidInput, "Unable to parse string as char")
        })?;

        Ok(Piece::from(c))
    }
}

impl TryFrom<u8> for Piece {
    type Error = BoardError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 13 {
            return Err(BoardError::new(ErrorKind::OutOfBounds, "Index too high"));
        }
        Ok(match value & TYPE_MASK {
            0 => Piece::King(value >> 3 == 0),
            1 => Piece::Queen(value >> 3 == 0),
            2 => Piece::Bishop(value >> 3 == 0),
            3 => Piece::Knight(value >> 3 == 0),
            4 => Piece::Rook(value >> 3 == 0),
            5 => Piece::Pawn(value >> 3 == 0),
            _ => Piece::Empty,
        })
    }
}

impl From<char> for Piece {
    fn from(c: char) -> Self {
        match c.to_ascii_lowercase() {
            'k' => Piece::King(c.is_uppercase()),
            'q' => Piece::Queen(c.is_uppercase()),
            'b' => Piece::Bishop(c.is_uppercase()),
            'n' => Piece::Knight(c.is_uppercase()),
            'r' => Piece::Rook(c.is_uppercase()),
            'p' => Piece::Pawn(c.is_uppercase()),
            _ => Piece::Empty,
        }
    }
}

impl Piece {
    pub fn is_white(&self) -> bool {
        *match self {
            Piece::King(is_white) => is_white,
            Piece::Queen(is_white) => is_white,
            Piece::Bishop(is_white) => is_white,
            Piece::Knight(is_white) => is_white,
            Piece::Rook(is_white) => is_white,
            Piece::Pawn(is_white) => is_white,
            Piece::Empty => &false,
        }
    }

    pub fn index(self: &Piece) -> usize {
        match self {
            Piece::King(is_white) => add_color(0, *is_white),
            Piece::Queen(is_white) => add_color(1, *is_white),
            Piece::Bishop(is_white) => add_color(2, *is_white),
            Piece::Knight(is_white) => add_color(3, *is_white),
            Piece::Rook(is_white) => add_color(4, *is_white),
            Piece::Pawn(is_white) => add_color(5, *is_white),
            Piece::Empty => 6,
        }
    }
}

fn add_color(index: usize, is_white: bool) -> usize {
    if !is_white {
        index | BLACK
    } else {
        index
    }
}

#[cfg(test)]
mod tests {
    use crate::piece::Piece;

    #[test]
    fn test_parse() {
        assert_eq!(Piece::King(true), "K".parse().unwrap())
    }
}
