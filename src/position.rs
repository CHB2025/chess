use std::{fmt, str};

use crate::error::{BoardError, ErrorKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position(u8);

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col = self.0 & 0b111;
        let row = self.0 >> 3;
        write!(f, "{}{}", char::from(b'a' + col), 8 - row)
    }
}

impl str::FromStr for Position {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_valid_position(s) {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Invalid position format",
            ));
        }
        let p_bytes = s.as_bytes();
        return Ok(Position((p_bytes[0] - b'a') + ((b'8' - p_bytes[1]) << 3)));
    }
}

impl TryFrom<u8> for Position {
    type Error = BoardError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 63 {
            return Err(BoardError::new(
                ErrorKind::OutOfBounds,
                "Index out of the board",
            ));
        }

        Ok(Position(value))
    }
}

impl Position {
    pub fn index(self) -> u8 {
        self.0
    }
}

fn is_valid_position(position: &str) -> bool {
    let p_bytes = position.as_bytes();
    if position.len() != 2 || p_bytes.len() != 2 {
        return false;
    }

    return b'a' <= p_bytes[0] && p_bytes[0] <= b'h' && b'0' <= p_bytes[1] && p_bytes[1] <= b'9';
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::position::Position;

    #[test]
    fn test_position_from_bit_index() {
        assert_eq!("a1", Position(56).to_string());
        assert_eq!("d4", Position(35).to_string());
        assert!(Position::try_from(68).is_err());
    }

    #[test]
    fn test_bit_index_from_position() {
        assert_eq!(Position(0), "a8".parse().unwrap());
        assert_eq!(Position(35), "d4".parse().unwrap());
        assert!("j3".parse::<Position>().is_err());
    }

    #[test]
    fn test_from_and_to_string() {
        assert_eq!("a1", Position::from_str("a1").unwrap().to_string().as_str());
    }

    #[test]
    fn board_layout() {
        for rank in 0..8 {
            print!("|");
            for file in 0..8 {
                let p = Position(file + (rank << 3));
                print!(" {p}({}) |", file + (rank << 3));
            }
            println!("");
        }
    }
}
