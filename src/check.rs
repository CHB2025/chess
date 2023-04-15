use serde::{Serialize, Deserialize};

use crate::Square;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Check {
    None,
    Single(Square),
    Double,
}
