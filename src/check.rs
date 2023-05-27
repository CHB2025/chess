#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Square, Bitboard};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Check {
    None,
    Single(Square),
    Double(Bitboard),
}
