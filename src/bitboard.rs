#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

use crate::{Dir, Ray, Square};

pub const ALL: Bitboard = Bitboard(!0);
pub const EMPTY: Bitboard = Bitboard(0);
pub const NOT_A_FILE: Bitboard = Bitboard(0x7f7f7f7f7f7f7f7f);
pub const NOT_H_FILE: Bitboard = Bitboard(0xfefefefefefefefe);

// Should make new method instead of public access to value
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn new(initial: u64) -> Bitboard {
        Bitboard(initial)
    }

    #[inline(always)]
    pub fn first_square(&self) -> Option<Square> {
        self.0.trailing_zeros().try_into().ok()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn between(sq1: Square, sq2: Square) -> Bitboard {
        match Ray::from(sq1, sq2) {
            Some(r) => {
                let mut found = false;
                r.into_iter().fold(EMPTY, |bb, sqr| {
                    if sqr == sq2 {
                        found = true;
                    }
                    if !found {
                        bb | sqr.into()
                    } else {
                        bb
                    }
                })
            }
            None => EMPTY,
        }
    }

    #[inline(always)]
    pub fn count_squares(&self) -> u32 {
        self.0.count_ones()
    }

    #[inline(always)]
    pub fn contains(&self, sq: Square) -> bool {
        *self & sq.into() == sq.into()
    }
}

pub struct BitboardIter(u64);

impl Iterator for BitboardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let sqr: Square = self.0.trailing_zeros().try_into().ok()?;
        self.0 ^= 1 << sqr.index();
        Some(sqr)
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;

    type IntoIter = BitboardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIter(self.0)
    }
}

impl FromIterator<Square> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Bitboard(0), |bb, sqr| bb | sqr.into())
    }
}

impl From<Square> for Bitboard {
    fn from(sqr: Square) -> Self {
        Bitboard(1 << sqr.index())
    }
}

impl From<Ray> for Bitboard {
    fn from(ray: Ray) -> Self {
        ray.into_iter().collect()
    }
}

// Bit And
impl BitAnd<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign<Bitboard> for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

// Bit Or
impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

// Bit Xor
impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

// Bit Shifting
impl Shl<usize> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl ShlAssign<usize> for Bitboard {
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs
    }
}

impl Shr<usize> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl ShrAssign<usize> for Bitboard {
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs
    }
}

impl Shl<Dir> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: Dir) -> Self::Output {
        (if rhs.offset().is_positive() {
            Bitboard(self.0 << rhs.offset())
        } else {
            Bitboard(self.0 >> rhs.offset().abs())
        }) & rhs.filter()
    }
}

impl ShlAssign<Dir> for Bitboard {
    fn shl_assign(&mut self, rhs: Dir) {
        if rhs.offset().is_positive() {
            self.0 <<= rhs.offset()
        } else {
            self.0 >>= rhs.offset().abs()
        }

        self.0 &= rhs.filter().0;
    }
}

impl Shr<Dir> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: Dir) -> Self::Output {
        (if rhs.offset().is_positive() {
            Bitboard(self.0 >> rhs.offset())
        } else {
            Bitboard(self.0 << rhs.offset().abs())
        }) & rhs.filter()
    }
}

impl ShrAssign<Dir> for Bitboard {
    fn shr_assign(&mut self, rhs: Dir) {
        if rhs.offset().is_positive() {
            self.0 >>= rhs.offset()
        } else {
            self.0 <<= rhs.offset().abs()
        }
        self.0 &= rhs.filter().0;
    }
}

impl Shl<i32> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: i32) -> Self::Output {
        if rhs.is_positive() {
            Bitboard(self.0 << rhs)
        } else {
            Bitboard(self.0 >> rhs.abs())
        }
    }
}

impl ShlAssign<i32> for Bitboard {
    fn shl_assign(&mut self, rhs: i32) {
        if rhs.is_positive() {
            self.0 <<= rhs
        } else {
            self.0 >>= rhs.abs()
        }
    }
}

impl Shr<i32> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: i32) -> Self::Output {
        if rhs.is_positive() {
            Bitboard(self.0 >> rhs)
        } else {
            Bitboard(self.0 << rhs.abs())
        }
    }
}

impl ShrAssign<i32> for Bitboard {
    fn shr_assign(&mut self, rhs: i32) {
        if rhs.is_positive() {
            self.0 >>= rhs
        } else {
            self.0 <<= rhs.abs()
        }
    }
}
