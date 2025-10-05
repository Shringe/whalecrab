use std::fmt::{self};

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
