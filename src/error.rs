use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidInput,
    OutOfBounds,
}

#[derive(Debug, Clone)]
pub struct BoardError {
    kind: ErrorKind,
    message: Option<String>,
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

impl BoardError {
    pub fn new(kind: ErrorKind, message: impl ToString) -> BoardError {
        BoardError {
            kind,
            message: Some(message.to_string()),
        }
    }
}
