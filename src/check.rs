use crate::Square;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Check {
    None,
    Single(Square),
    Double,
}
