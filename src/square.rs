#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt, str};

use crate::{BoardError, Dir, ErrorKind, Bitboard};

const A1: u8 = 63;

pub mod squares {
    use super::Square;

    pub const A1: Square = Square(63);
    pub const B1: Square = Square(62);
    pub const C1: Square = Square(61);
    pub const D1: Square = Square(60);
    pub const E1: Square = Square(59);
    pub const F1: Square = Square(58);
    pub const G1: Square = Square(57);
    pub const H1: Square = Square(56);

    pub const A2: Square = Square(55);
    pub const B2: Square = Square(54);
    pub const C2: Square = Square(53);
    pub const D2: Square = Square(52);
    pub const E2: Square = Square(51);
    pub const F2: Square = Square(50);
    pub const G2: Square = Square(49);
    pub const H2: Square = Square(48);

    pub const A3: Square = Square(47);
    pub const B3: Square = Square(46);
    pub const C3: Square = Square(45);
    pub const D3: Square = Square(44);
    pub const E3: Square = Square(43);
    pub const F3: Square = Square(42);
    pub const G3: Square = Square(41);
    pub const H3: Square = Square(40);

    pub const A4: Square = Square(39);
    pub const B4: Square = Square(38);
    pub const C4: Square = Square(37);
    pub const D4: Square = Square(36);
    pub const E4: Square = Square(35);
    pub const F4: Square = Square(34);
    pub const G4: Square = Square(33);
    pub const H4: Square = Square(32);

    pub const A5: Square = Square(31);
    pub const B5: Square = Square(30);
    pub const C5: Square = Square(29);
    pub const D5: Square = Square(28);
    pub const E5: Square = Square(27);
    pub const F5: Square = Square(26);
    pub const G5: Square = Square(25);
    pub const H5: Square = Square(24);

    pub const A6: Square = Square(23);
    pub const B6: Square = Square(22);
    pub const C6: Square = Square(21);
    pub const D6: Square = Square(20);
    pub const E6: Square = Square(19);
    pub const F6: Square = Square(18);
    pub const G6: Square = Square(17);
    pub const H6: Square = Square(16);

    pub const A7: Square = Square(15);
    pub const B7: Square = Square(14);
    pub const C7: Square = Square(13);
    pub const D7: Square = Square(12);
    pub const E7: Square = Square(11);
    pub const F7: Square = Square(10);
    pub const G7: Square = Square(9);
    pub const H7: Square = Square(8);

    pub const A8: Square = Square(7);
    pub const B8: Square = Square(6);
    pub const C8: Square = Square(5);
    pub const D8: Square = Square(4);
    pub const E8: Square = Square(3);
    pub const F8: Square = Square(2);
    pub const G8: Square = Square(1);
    pub const H8: Square = Square(0);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Square(u8);

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
        if !is_valid_square(s) {
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
    #[inline(always)]
    pub fn index(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn rank(&self) -> u8 {
        self.0 >> 3
    }
    #[inline(always)]
    pub fn file(&self) -> u8 {
        self.0 & 7
    }
    #[inline(always)]
    pub fn diagonal(&self) -> u8 {
        self.rank().wrapping_sub(self.file()) & 15
    }
    #[inline(always)]
    pub fn anti_diagonal(&self) -> u8 {
        (self.rank() + self.file()) ^ 7
    }

    /// Returns the next square in the dir if it is on the board
    ///
    /// # Examples
    /// ```
    /// # use chb_chess::{Square, Dir, squares};
    ///
    /// let origin = squares::A1;
    /// let added = origin.checked_add(Dir::NorEast);
    ///
    /// assert_eq!(added, Some(squares::B2));
    /// ```
    #[inline(always)]
    pub fn checked_add(self, dir: Dir) -> Option<Self> {
        (Bitboard::from(self) << dir).first_square()
    }
}

fn is_valid_square(square: &str) -> bool {
    let p_bytes = square.as_bytes();
    if square.len() != 2 || p_bytes.len() != 2 {
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
