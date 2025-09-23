use crate::file::{File, ALL_FILES};
use crate::rank::{Rank, ALL_RANKS};
use crate::square::*;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not};

/// A good old-fashioned bitboard
/// You *do* have access to the actual value, but you are probably better off
/// using the implemented operators to work with this object.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash, Debug)]
pub struct BitBoard(pub u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        let mut ranks = ALL_RANKS.clone();
        let mut files = ALL_FILES.clone();
        ranks.reverse();
        files.reverse();

        for r in ranks {
            for f in files {
                let sqbb = BitBoard::from_rank_file(r, f);
                if self.has_square(&sqbb) {
                    out.push_str("1 ");
                } else {
                    out.push_str("0 ");
                }
            }

            out.push_str(format!("| {}", r).as_str());
            out.push('\n');
        }

        out.push_str("---------------\n");
        out.push_str("A B C D E F G H");

        write!(f, "{}", out)
    }
}

/// An empty bitboard.  It is sometimes useful to use !EMPTY to get the universe of squares.
pub const EMPTY: BitBoard = BitBoard(0);

// Impl BitAnd
impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

// Impl BitOr
impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

// Impl BitXor

impl BitXor for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

// Impl BitAndAssign

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: BitBoard) {
        self.0 &= other.0;
    }
}

impl BitAndAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: &BitBoard) {
        self.0 &= other.0;
    }
}

// Impl BitOrAssign
impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: BitBoard) {
        self.0 |= other.0;
    }
}

impl BitOrAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: &BitBoard) {
        self.0 |= other.0;
    }
}

// Impl BitXor Assign
impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: BitBoard) {
        self.0 ^= other.0;
    }
}

impl BitXorAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: &BitBoard) {
        self.0 ^= other.0;
    }
}

// Impl Mul
impl Mul for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

// Impl Not
impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl Not for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl BitBoard {
    pub const INITIAL_BLACK_PAWN: BitBoard =
        BitBoard(0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000);
    pub const INITIAL_BLACK_KNIGHT: BitBoard =
        BitBoard(0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
    pub const INITIAL_BLACK_BISHOP: BitBoard =
        BitBoard(0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
    pub const INITIAL_BLACK_ROOK: BitBoard =
        BitBoard(0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
    pub const INITIAL_BLACK_QUEEN: BitBoard =
        BitBoard(0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
    pub const INITIAL_BLACK_KING: BitBoard =
        BitBoard(0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

    pub const INITIAL_WHITE_PAWN: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000);
    pub const INITIAL_WHITE_KNIGHT: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010);
    pub const INITIAL_WHITE_BISHOP: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100);
    pub const INITIAL_WHITE_ROOK: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001);
    pub const INITIAL_WHITE_QUEEN: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000);
    pub const INITIAL_WHITE_KING: BitBoard =
        BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000);

    /// Construct a new bitboard from a u64
    #[inline]
    pub fn new(b: u64) -> BitBoard {
        BitBoard(b)
    }

    /// Construct a new `BitBoard` with a particular `Square` set
    #[inline]
    pub fn from_rank_file(rank: Rank, file: File) -> BitBoard {
        BitBoard::from_square(Square::make_square(rank, file))
    }

    /// Set a square on a BitBoard
    pub fn set(&mut self, sq: Square) {
        *self |= BitBoard::from_square(sq);
    }

    /// ands a vector of squares together into a bitboard
    pub fn from_square_vec(squares: Vec<Square>) -> BitBoard {
        squares
            .into_iter()
            .map(BitBoard::from_square)
            .fold(EMPTY, |acc, bb| acc | bb)
    }

    /// Construct a new `BitBoard` with a particular `Square` set
    #[inline]
    pub fn from_square(sq: Square) -> BitBoard {
        BitBoard(1u64 << sq.to_int())
    }

    /// Convert an `Option<Square>` to an `Option<BitBoard>`
    #[inline]
    pub fn from_maybe_square(sq: Option<Square>) -> Option<BitBoard> {
        sq.map(BitBoard::from_square)
    }

    /// Convert a `BitBoard` to a `Square`.  This grabs the least-significant `Square`
    #[inline]
    pub fn to_square(&self) -> Square {
        Square::new(self.0.trailing_zeros() as u8)
    }

    /// Check if a square's index is on in the bitboard
    /// The BitBoard should only have a single square on
    pub fn has_square(&self, sqbb: &BitBoard) -> bool {
        self & sqbb != EMPTY
    }

    /// Count the number of `Squares` set in this `BitBoard`
    #[inline]
    pub fn popcnt(&self) -> u32 {
        self.0.count_ones()
    }

    /// Reverse this `BitBoard`.  Look at it from the opponents perspective.
    #[inline]
    pub fn reverse_colors(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
    }

    /// Convert this `BitBoard` to a `usize` (for table lookups)
    #[inline]
    pub fn to_size(&self, rightshift: u8) -> usize {
        (self.0 >> rightshift) as usize
    }
}

/// For the `BitBoard`, iterate over every `Square` set.
impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let result = self.to_square();
            *self ^= BitBoard::from_square(result);
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    use super::*;

    #[test]
    fn has_squares() {
        let board = Board::default();
        let occupied = board.occupied_bitboard();

        let first = BitBoard::from_square(Square::C7);
        let second = BitBoard::from_square(Square::H1);
        let empty = BitBoard::from_square(Square::G3);

        assert!(occupied.has_square(&first));
        assert!(occupied.has_square(&second));
        assert!(!occupied.has_square(&empty));
    }

    #[test]
    fn display_formatting() {
        let board = Board::default();
        let occupied = board.occupied_black_bitboard();
        let looking_for = "1 1 1 1 1 1 1 1 | 8
1 1 1 1 1 1 1 1 | 7
0 0 0 0 0 0 0 0 | 6
0 0 0 0 0 0 0 0 | 5
0 0 0 0 0 0 0 0 | 4
0 0 0 0 0 0 0 0 | 3
0 0 0 0 0 0 0 0 | 2
0 0 0 0 0 0 0 0 | 1
---------------
A B C D E F G H"
            .to_string();
        let out = occupied.to_string();

        println!("{}", out);
        println!("{}", looking_for);
        assert_eq!(out, looking_for);
    }
}
