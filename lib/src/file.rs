use crate::bitboard::BitBoard;

/// Describe a file (column) on a chess board
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

/// How many files are there?
pub const NUM_FILES: usize = 8;

/// Enumerate all files
pub const ALL_FILES: [File; NUM_FILES] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

impl File {
    /// # Safety
    /// This function should only be called with a value known to be from 0-7
    pub const unsafe fn from_int_unchecked(val: u8) -> File {
        unsafe { std::mem::transmute::<u8, File>(val) }
    }

    pub const fn from_int(val: u8) -> Option<File> {
        if val > 7 {
            None
        } else {
            unsafe { Some(File::from_int_unchecked(val)) }
        }
    }

    pub const fn to_int(self) -> u8 {
        self as u8
    }

    /// Convert a `usize` into a `File` (the inverse of to_index).  If i > 7, wrap around.
    #[inline]
    pub fn from_index(i: usize) -> File {
        // match is optimized to no-op with opt-level=1 with rustc 1.53.0
        match i & 7 {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => unreachable!(),
        }
    }

    /// Converts a char into the correct file if possible
    #[inline]
    pub fn from_char(c: char) -> Option<File> {
        match c {
            'a' => Some(File::A),
            'b' => Some(File::B),
            'c' => Some(File::C),
            'd' => Some(File::D),
            'e' => Some(File::E),
            'f' => Some(File::F),
            'g' => Some(File::G),
            'h' => Some(File::H),
            _ => None,
        }
    }

    /// Creates a bitboard with the entire file set
    #[inline]
    pub fn mask(self) -> BitBoard {
        BitBoard::new(0x0101010101010101 << self as u8)
    }

    /// Go one file to the left.  If impossible, wrap around.
    #[inline]
    pub fn left(&self) -> File {
        File::from_index(self.to_index().wrapping_sub(1))
    }

    /// Go one file to the right.  If impossible, wrap around.
    #[inline]
    pub fn right(&self) -> File {
        File::from_index(self.to_index() + 1)
    }

    /// Convert this `File` into a `usize` from 0 to 7 inclusive.
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bitboard() {
        let file = File::C;
        let expected = BitBoard::new(
            0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100,
        );
        assert_eq!(file.mask(), expected);
    }
}
