use std::{fmt, str};

use crate::{Bitboard, Dir, BoardError, ErrorKind};

const A1: u8 = 63;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct Square(pub(super) u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col = self.0 & 0b111;
        let row = self.0 >> 3;
        write!(f, "{}{}", char::from(b'h' - col), 8 - row)
    }
}

impl str::FromStr for Square {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_valid_position(s) {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Invalid position format",
            ));
        }
        let p_bytes = s.as_bytes();
        Ok(Square((b'h' - p_bytes[0]) + ((b'8' - p_bytes[1]) << 3)))
    }
}

impl TryFrom<u8> for Square {
    type Error = BoardError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > A1 {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of the board",
            ));
        }

        Ok(Square(value))
    }
}
impl TryFrom<usize> for Square {
    type Error = BoardError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > A1.into() {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of the board",
            ));
        }
        Ok(Square(value.try_into()?))
    }
}
impl TryFrom<u32> for Square {
    type Error = BoardError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > A1.into() {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of the board",
            ));
        }
        Ok(Square(value.try_into()?))
    }
}
impl TryFrom<u64> for Square {
    type Error = BoardError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > A1.into() {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of the board",
            ));
        }
        Ok(Square(value.try_into()?))
    }
}

impl Square {
    pub fn index(&self) -> u8 {
        self.0
    }

    pub fn mask(&self) -> Bitboard {
        1 << self.0
    }

    pub fn rank(&self) -> u8 {
        self.0 >> 3
    }
    pub fn file(&self) -> u8 {
        self.0 & 7
    }
    //Are these useful in any situation?
    // Maybe quickly calculating if check is possible by a given piece
    pub fn diagonal(&self) -> u8 {
        self.rank().wrapping_sub(self.file())  & 15 
    }
    pub fn anti_diagonal(&self) -> u8 {
        (self.rank() + self.file()) ^ 7
    }

    pub fn checked_add(&self, dir: Dir) -> Option<Self> {
        let new_mask = dir.shift(self.mask());
        match new_mask {
            0 => None,
            mask => Some(Self(63 - mask.leading_zeros() as u8))
        }
    }

}

fn is_valid_position(position: &str) -> bool {
    let p_bytes = position.as_bytes();
    if position.len() != 2 || p_bytes.len() != 2 {
        return false;
    }

    b'a' <= p_bytes[0] && p_bytes[0] <= b'h' && b'0' <= p_bytes[1] && p_bytes[1] <= b'8'
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::square::Square;

    #[test]
    fn test_position_from_bit_index() {
        assert_eq!("a1", Square(63).to_string());
        assert_eq!("d4", Square(36).to_string());
        assert!(Square::try_from(68_u8).is_err());
    }

    #[test]
    fn test_bit_index_from_position() {
        assert_eq!(Square(63), "a1".parse().unwrap());
        assert_eq!(Square(36), "d4".parse().unwrap());
        assert!("j3".parse::<Square>().is_err());
    }

    #[test]
    fn test_from_and_to_string() {
        assert_eq!("a1", Square::from_str("a1").unwrap().to_string().as_str());
    }

    #[test]
    fn board_layout() {
        for rank in 0..8 {
            print!("|");
            for file in 0..8 {
                let p = Square(file + (rank << 3));
                print!(" {p}({}) |", file + (rank << 3));
            }
            println!();
        }
    }
}
