use crate::{Bitboard, Dir, Piece, Position, Square};

pub struct Ray {
    pub origin: Square,
    pub dir: Dir,
}

impl Ray {
    /// Returns a bitboard representing the ray in a u64. Excludes the origin
    pub fn mask(&self) -> Bitboard {
        let mut result = 0u64;
        let mut cur = self.dir.shift(self.origin.mask());
        while cur != 0 {
            result |= cur;
            cur = self.dir.shift(cur)
        }
        result
    }

    /// Returns a vector of all the squares along a ray, excluding the origin
    pub fn squares(&self) -> Vec<Square> {
        let mut result: Vec<Square> = Vec::new();
        let mut next = self.origin.checked_add(self.dir);
        while let Some(cur) = next {
            result.push(cur);
            next = cur.checked_add(self.dir);
        }
        result
    }

    /// Returns a vector of the non-empty pieces along a ray
    pub fn pieces(&self, pos: &Position) -> Vec<Piece> {
        let mut result = Vec::new();
        let mut next = self.origin.checked_add(self.dir);
        while let Some(cur) = next {
            if pos[cur] != Piece::Empty {
                result.push(pos[cur]);
            }
            next = cur.checked_add(self.dir);
        }
        result
    }

    /// Returns a vector of the empty squares between the origin of the ray and
    /// the first piece in the vector or the edge of the board
    pub fn squares_to_piece(&self, pos: &Position) -> Vec<Square> {
        let mut result = Vec::new();
        let mut next = self.origin.checked_add(self.dir);
        while let Some(cur) = next {
            if pos[cur] == Piece::Empty {
                result.push(cur);
                next = cur.checked_add(self.dir);
            } else {
                next = None;
            }
        }
        result
    }

    /// Returns the Square occupied by the first piece that the ray hits
    pub fn first_piece(&self, pos: &Position) -> Option<Square> {
        let empty_squares = self.squares_to_piece(pos);
        if empty_squares.is_empty() {
            return self.origin.checked_add(self.dir);
        }
        let last = empty_squares[empty_squares.len() - 1];
        last.checked_add(self.dir)
    }

    pub fn contains(&self, square: Square) -> bool {
        let diff = square.index() as i32 - self.origin.index() as i32;
        //possible && diff.is_positive() == self.dir.offset().is_positive()
        diff.is_positive() == self.dir.offset().is_positive() && diff % self.dir.offset() == 0
    }

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
