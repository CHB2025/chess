use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Castle {
    Both,
    KingSide,
    QueenSide,
    None,
}

impl Display for Castle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Castle::Both => "kq",
            Castle::KingSide => "k",
            Castle::QueenSide => "q",
            Castle::None => "-",
        };
        write!(f, "{}", output)
    }
}

impl Castle {
    pub fn with_king_side(self, may_castle: bool) -> Self {
        match (self, may_castle) {
            (Castle::Both | Castle::QueenSide, true) => Castle::Both,
            (Castle::Both | Castle::QueenSide, false) => Castle::QueenSide,
            (Castle::KingSide | Castle::None, true) => Castle::KingSide,
            (Castle::KingSide | Castle::None, false) => Castle::None,
        }
    }

    pub fn with_queen_side(self, may_castle: bool) -> Self {
        match (self, may_castle) {
            (Castle::Both | Castle::KingSide, true) => Castle::Both,
            (Castle::Both | Castle::KingSide, false) => Castle::KingSide,
            (Castle::QueenSide | Castle::None, true) => Castle::QueenSide,
            (Castle::QueenSide | Castle::None, false) => Castle::None,
        }
    }

    pub fn get_king_side(self) -> bool {
        matches!(self, Castle::Both | Castle::KingSide)
    }

    pub fn get_queen_side(self) -> bool {
        matches!(self, Castle::Both | Castle::QueenSide)
    }
}
