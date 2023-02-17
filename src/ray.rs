mod iter;

use crate::{Dir, Square};

use self::iter::RayIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ray {
    pub origin: Square,
    pub dir: Dir,
}

impl IntoIterator for Ray {
    type IntoIter = RayIterator;
    type Item = Square;

    fn into_iter(self) -> Self::IntoIter {
        RayIterator {
            curr: self.origin,
            dir: self.dir,
        }
    }
}

impl Ray {
    pub fn contains(&self, square: Square) -> bool {
        let diff = square.index() as i32 - self.origin.index() as i32;
        //possible && diff.is_positive() == self.dir.offset().is_positive()
        diff.is_positive() == self.dir.offset().is_positive() && diff % self.dir.offset() == 0
    }

    #[inline(always)]
    pub fn from(origin: Square, other: Square) -> Option<Self> {
        if origin == other {
            return None;
        }
        let is_greater = origin.index() > other.index();
        let dir = if origin.file() == other.file() {
            if is_greater {
                Dir::North
            } else {
                Dir::South
            }
        } else if origin.rank() == other.rank() {
            if is_greater {
                Dir::East
            } else {
                Dir::West
            }
        } else if origin.diagonal() == other.diagonal() {
            if is_greater {
                Dir::NorEast
            } else {
                Dir::SouWest
            }
        } else if origin.anti_diagonal() == other.anti_diagonal() {
            if is_greater {
                Dir::NorWest
            } else {
                Dir::SouEast
            }
        } else {
            return None;
        };
        Some(Ray { origin, dir })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let r = Ray {
            origin: "d4".parse().unwrap(),
            dir: Dir::NorEast,
        };
        assert!(r.contains("f6".parse().unwrap()));
        assert!(!r.contains("c5".parse().unwrap()));
        assert!(!r.contains("b2".parse().unwrap()));
    }

    #[test]
    fn test_from() {
        let sq1: Square = "d4".parse().unwrap();
        let sq2: Square = "e5".parse().unwrap();
        let r = Ray::from(sq1, sq2).unwrap();
        assert_eq!(sq1, r.origin);
        assert!(r.contains(sq2));
    }
}
