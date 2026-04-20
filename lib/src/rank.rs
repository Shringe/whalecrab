use std::fmt::{self};

use crate::bitboard::BitBoard;

/// Describe a rank (row) on a chess board
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Rank {
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
    Eighth = 7,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_index() + 1)
    }
}

/// How many ranks are there?
pub const NUM_RANKS: usize = 8;

/// Enumerate all ranks
pub const ALL_RANKS: [Rank; NUM_RANKS] = [
    Rank::First,
    Rank::Second,
    Rank::Third,
    Rank::Fourth,
    Rank::Fifth,
    Rank::Sixth,
    Rank::Seventh,
    Rank::Eighth,
];

impl Rank {
    /// # Safety
    /// This function should only be called with a value known to be from 0-7
    pub const unsafe fn from_int_unchecked(val: u8) -> Rank {
        unsafe { std::mem::transmute::<u8, Rank>(val) }
    }

    pub const fn from_int(val: u8) -> Option<Rank> {
        if val > 7 {
            None
        } else {
            unsafe { Some(Rank::from_int_unchecked(val)) }
        }
    }

    pub const fn to_int(self) -> u8 {
        self as u8
    }

    /// Convert a `usize` into a `Rank` (the inverse of to_index).  If the number is > 7, wrap
    /// around.
    #[inline]
    pub fn from_index(i: usize) -> Rank {
        // match is optimized to no-op with opt-level=1 with rustc 1.53.0
        match i & 7 {
            0 => Rank::First,
            1 => Rank::Second,
            2 => Rank::Third,
            3 => Rank::Fourth,
            4 => Rank::Fifth,
            5 => Rank::Sixth,
            6 => Rank::Seventh,
            7 => Rank::Eighth,
            _ => unreachable!(),
        }
    }

    /// Creates a bitboard with the entire rank set
    #[inline]
    pub const fn mask(self) -> BitBoard {
        BitBoard::new(0xFF << (self as u8 * 8))
    }

    /// Go one rank down.  If impossible, wrap around.
    #[inline]
    pub fn down(&self) -> Rank {
        Rank::from_index(self.to_index().wrapping_sub(1))
    }

    /// Go one file up.  If impossible, wrap around.
    #[inline]
    pub fn up(&self) -> Rank {
        Rank::from_index(self.to_index() + 1)
    }

    /// Convert this `Rank` into a `usize` between 0 and 7 (inclusive).
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_mask() {
        let rank = Rank::Seventh;
        let expected = BitBoard::INITIAL_BLACK_PAWNS;
        let actual = rank.mask();
        assert_eq!(actual, expected);
    }
}
