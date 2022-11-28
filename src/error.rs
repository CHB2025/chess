use std::{
    char::ParseCharError,
    fmt,
    num::{self, ParseIntError},
};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidInput,
    OutOfBounds,
}

#[derive(Debug, Clone)]
pub struct BoardError {
    pub kind: ErrorKind,
    pub message: Option<String>,
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err = match self.kind {
            ErrorKind::InvalidInput => "Invalid Input",
            ErrorKind::OutOfBounds => "Out of Bounds",
        }
        .to_string();

        if let Some(m) = &self.message {
            err += &format!(": {}", m);
        }

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
    pub fn new(kind: ErrorKind, message: impl ToString) -> BoardError {
        BoardError {
            kind,
            message: Some(message.to_string()),
        }
    }
}
