#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Square;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Check {
    None,
    Single(Square),
    Double,
}
