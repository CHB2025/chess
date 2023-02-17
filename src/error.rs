use std::{
    char::ParseCharError,
    fmt,
    num::{self, ParseIntError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardError {
    pub kind: ErrorKind,
    pub message: &'static str,
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err = match self.kind {
            ErrorKind::InvalidInput => "Invalid Input",
            ErrorKind::OutOfBounds => "Out of Bounds",
        }
        .to_string();

        err += &format!(": {}", self.message);

        write!(f, "{}", err)
    }
}

impl From<num::TryFromIntError> for BoardError {
    fn from(_: num::TryFromIntError) -> Self {
        BoardError::new(ErrorKind::OutOfBounds, "Invalid integer conversion")
    }
}

impl From<ParseIntError> for BoardError {
    fn from(_: ParseIntError) -> Self {
        BoardError::new(ErrorKind::InvalidInput, "Unable to parse integer")
    }
}

impl From<ParseCharError> for BoardError {
    fn from(_: ParseCharError) -> Self {
        BoardError::new(ErrorKind::InvalidInput, "Unable to parse char")
    }
}

impl BoardError {
    pub fn new(kind: ErrorKind, message: &'static str) -> BoardError {
        BoardError {
            kind,
            message,
        }
    }
}
