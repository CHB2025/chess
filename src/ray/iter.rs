use crate::{Square, Dir};

pub struct RayIterator {
    pub(super) curr: Square,
    pub(super) dir: Dir,
}

impl Iterator for RayIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.curr.checked_add(self.dir)?;
        self.curr = next;
        Some(next)
    }
}
