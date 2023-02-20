use crate::Square;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Check {
    None,
    Single(Square),
    Double,
}
