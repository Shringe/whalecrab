use std::fmt::{self, Display};
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::board::{Board, Color, PieceType};
use crate::file::File;
use crate::rank::Rank;

pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Enumerate all directions
pub const ALL_DIRECTIONS: [Direction; 8] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
    Direction::NorthEast,
    Direction::NorthWest,
    Direction::SouthEast,
    Direction::SouthWest,
];

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct Square(u8);

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{}", self.get_file(), self.get_rank())
    }
}

impl FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let f = chars.next().ok_or(())?;
        let r = chars.next().ok_or(())?;
        let rank = Rank::from_index(r.to_digit(10).ok_or(())? as usize - 1);
        let file = File::from_char(f).ok_or(())?;
        Ok(Self::make_square(rank, file))
    }
}

impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);
    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);
    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);
    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);
    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);
    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);
    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);
    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    pub fn new(sq: u8) -> Square {
        Square(sq & 63)
    }

    pub fn to_int(&self) -> u8 {
        self.0
    }

    pub fn get_rank(&self) -> Rank {
        Rank::from_index((self.0 >> 3) as usize)
    }

    pub fn get_file(&self) -> File {
        File::from_index((self.0 & 7) as usize)
    }

    pub fn make_square(rank: Rank, file: File) -> Square {
        Square(((rank.to_index() as u8) << 3) ^ (file.to_index() as u8))
    }

    pub fn uleft(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth || self.get_file() == File::A {
            None
        } else {
            Some(Square::make_square(
                self.get_rank().up(),
                self.get_file().left(),
            ))
        }
    }

    pub fn uright(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth || self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(
                self.get_rank().up(),
                self.get_file().right(),
            ))
        }
    }

    pub fn dleft(&self) -> Option<Square> {
        if self.get_rank() == Rank::First || self.get_file() == File::A {
            None
        } else {
            Some(Square::make_square(
                self.get_rank().down(),
                self.get_file().left(),
            ))
        }
    }

    pub fn dright(&self) -> Option<Square> {
        if self.get_rank() == Rank::First || self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(
                self.get_rank().down(),
                self.get_file().right(),
            ))
        }
    }

    pub fn up(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth {
            None
        } else {
            Some(Square::make_square(self.get_rank().up(), self.get_file()))
        }
    }

    pub fn down(&self) -> Option<Square> {
        if self.get_rank() == Rank::First {
            None
        } else {
            Some(Square::make_square(self.get_rank().down(), self.get_file()))
        }
    }

    pub fn left(&self) -> Option<Square> {
        if self.get_file() == File::A {
            None
        } else {
            Some(Square::make_square(self.get_rank(), self.get_file().left()))
        }
    }

    pub fn right(&self) -> Option<Square> {
        if self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(
                self.get_rank(),
                self.get_file().right(),
            ))
        }
    }

    pub fn backward(&self, color: &Color) -> Option<Square> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up(),
        }
    }

    pub fn forward(&self, color: &Color) -> Option<Square> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down(),
        }
    }

    /// forward left
    pub fn fleft(&self, color: &Color) -> Option<Square> {
        match color {
            Color::White => self.uleft(),
            Color::Black => self.dleft(),
        }
    }

    /// forward right
    pub fn fright(&self, color: &Color) -> Option<Square> {
        match color {
            Color::White => self.uright(),
            Color::Black => self.dright(),
        }
    }

    /// Consumes the square and determines if it is on it the given bitboard
    pub fn in_bitboard(&self, bb: &BitBoard) -> bool {
        bb.has_square(&BitBoard::from_square(*self))
    }

    /// Moves one square in a direction. Useful for ray pieces.
    pub fn walk(&self, direction: &Direction) -> Option<Square> {
        match direction {
            Direction::North => self.up(),
            Direction::South => self.down(),
            Direction::East => self.right(),
            Direction::West => self.left(),
            Direction::NorthEast => self.uright(),
            Direction::NorthWest => self.uleft(),
            Direction::SouthEast => self.dright(),
            Direction::SouthWest => self.dleft(),
        }
    }

    /// Generates a ray of squares until eiher the end of the board, right before a friendly piece,
    /// or it ends right on an enemy piece. Used for ray pieces in move generation.
    pub fn ray(&self, direction: &Direction, board: &Board) -> Vec<Square> {
        let mut squares = Vec::new();
        let enemy = board.turn.opponent();

        let mut current = *self;

        while let Some(forward) = current.walk(direction) {
            if let Some(color) = board.determine_color(forward) {
                let is_enemy = color == enemy;
                let is_king = board.determine_piece(forward) == Some(PieceType::King);
                if is_enemy {
                    squares.push(forward);
                }

                if !(is_king && is_enemy) {
                    break;
                }
            } else {
                squares.push(forward);
            }

            current = forward;
        }

        squares
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    use super::*;

    #[test]
    fn in_bitboards() {
        let board = Board::default();
        let occupied = board.occupied_bitboard();

        let first = Square::C7;
        let second = Square::H1;
        let empty = Square::G3;

        assert!(first.in_bitboard(&occupied));
        assert!(second.in_bitboard(&occupied));
        assert!(!empty.in_bitboard(&occupied));
    }

    #[test]
    fn uright_equals_up_right() {
        let sq = Square::E4;
        assert_eq!(sq.up().unwrap().right(), sq.uright());
    }

    #[test]
    fn dleft_eqauls_down_left() {
        let sq = Square::F7;
        assert_eq!(sq.down().unwrap().left(), sq.dleft());
    }

    #[test]
    fn display_format() {
        let a6 = Square::A6;
        let g3 = Square::G3;
        assert_eq!("A6", format!("{}", a6));
        assert_eq!("G3", format!("{}", g3));
    }
}
