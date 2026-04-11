use std::fmt::{self, Display};
use std::str::FromStr;

use crate::bitboard::{BitBoard, EMPTY};
use crate::file::File;
use crate::implement_operations;
use crate::movegen::moves::Move;
use crate::movegen::pieces::piece::{PieceColor, PieceMoveInfo, PieceType};
use crate::position::game::Game;
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

#[derive(Debug, PartialEq)]
pub enum SquareParseError {
    EmptyInput,
    MissingRank,
    InvalidRank(char),
    InvalidFile(char),
}

impl fmt::Display for SquareParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquareParseError::EmptyInput => write!(f, "input was empty"),
            SquareParseError::MissingRank => write!(f, "missing rank digit"),
            SquareParseError::InvalidRank(c) => write!(f, "invalid rank '{c}', expected 1-8"),
            SquareParseError::InvalidFile(c) => write!(f, "invalid file '{c}', expected a-h"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct Square(u8);

implement_operations!(Square, Self, u8, [Add, Sub]);

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{}", self.get_file(), self.get_rank())
    }
}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let f = chars.next().ok_or(SquareParseError::EmptyInput)?;
        let r = chars.next().ok_or(SquareParseError::MissingRank)?;
        let rank =
            Rank::from_index(r.to_digit(10).ok_or(SquareParseError::InvalidRank(r))? as usize - 1);
        let file = File::from_char(f).ok_or(SquareParseError::InvalidFile(f))?;
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

    pub const fn new(sq: u8) -> Square {
        Square(sq & 63)
    }

    /// # Safety
    /// `sq` should be less than 64
    pub const unsafe fn new_unchecked(sq: u8) -> Square {
        Square(sq)
    }

    pub const fn to_int(&self) -> u8 {
        self.0
    }

    pub fn get_rank(&self) -> Rank {
        unsafe { Rank::from_int_unchecked(self.0 >> 3) }
    }

    pub fn get_file(&self) -> File {
        unsafe { File::from_int_unchecked(self.0 & 7) }
    }

    pub fn make_square(rank: Rank, file: File) -> Square {
        Square(((rank.to_index() as u8) << 3) ^ (file.to_index() as u8))
    }

    /// Flips the side of the square for the opposite color
    pub const fn flip_side(&self) -> Square {
        Square::new(self.0 ^ 56)
    }

    /// # Safety
    /// `self.get_file() > File::A && self.get_rank() < Rank::Eighth`
    pub const unsafe fn uleft_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 + 7) }
    }

    /// # Safety
    /// `self.get_file() < File::H && self.get_rank() < Rank::Eighth`
    pub const unsafe fn uright_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 + 9) }
    }

    /// # Safety
    /// `self.get_file() > File::A && self.get_rank() > Rank::First`
    pub const unsafe fn dleft_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 - 9) }
    }

    /// # Safety
    /// `self.get_file() < File::H && self.get_rank() > Rank::First`
    pub const unsafe fn dright_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 - 7) }
    }

    /// # Safety
    /// `self.get_rank() < Rank::Eighth`
    pub const unsafe fn up_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 + 8) }
    }

    /// # Safety
    /// `self.get_rank() > Rank::First`
    pub const unsafe fn down_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 - 8) }
    }

    /// # Safety
    /// `self.get_file() > File::A`
    pub const unsafe fn left_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 - 1) }
    }

    /// # Safety
    /// `self.get_file() < File::H`
    pub const unsafe fn right_unchecked(&self) -> Square {
        unsafe { Square::new_unchecked(self.0 + 1) }
    }

    pub const fn uleft(&self) -> Option<Square> {
        if self.0 >= 56 || self.0 & 7 == 0 {
            None
        } else {
            Some(unsafe { self.uleft_unchecked() })
        }
    }

    pub const fn uright(&self) -> Option<Square> {
        if self.0 >= 56 || self.0 & 7 == 7 {
            None
        } else {
            Some(unsafe { self.uright_unchecked() })
        }
    }

    pub const fn dleft(&self) -> Option<Square> {
        if self.0 < 8 || self.0 & 7 == 0 {
            None
        } else {
            Some(unsafe { self.dleft_unchecked() })
        }
    }

    pub const fn dright(&self) -> Option<Square> {
        if self.0 < 8 || self.0 & 7 == 7 {
            None
        } else {
            Some(unsafe { self.dright_unchecked() })
        }
    }

    pub const fn up(&self) -> Option<Square> {
        if self.0 >= 56 {
            None
        } else {
            Some(unsafe { self.up_unchecked() })
        }
    }

    pub const fn down(&self) -> Option<Square> {
        if self.0 < 8 {
            None
        } else {
            Some(unsafe { self.down_unchecked() })
        }
    }

    pub const fn left(&self) -> Option<Square> {
        if self.0 & 7 == 0 {
            None
        } else {
            Some(unsafe { self.left_unchecked() })
        }
    }

    pub const fn right(&self) -> Option<Square> {
        if self.0 & 7 == 7 {
            None
        } else {
            Some(unsafe { self.right_unchecked() })
        }
    }

    pub fn backward(&self, color: &PieceColor) -> Option<Square> {
        match color {
            PieceColor::White => self.down(),
            PieceColor::Black => self.up(),
        }
    }

    pub fn forward(&self, color: &PieceColor) -> Option<Square> {
        match color {
            PieceColor::White => self.up(),
            PieceColor::Black => self.down(),
        }
    }

    /// forward left
    pub fn fleft(&self, color: &PieceColor) -> Option<Square> {
        match color {
            PieceColor::White => self.uleft(),
            PieceColor::Black => self.dleft(),
        }
    }

    /// forward right
    pub fn fright(&self, color: &PieceColor) -> Option<Square> {
        match color {
            PieceColor::White => self.uright(),
            PieceColor::Black => self.dright(),
        }
    }

    /// Consumes the square and determines if it is on it the given bitboard
    pub fn in_bitboard(&self, bb: &BitBoard) -> bool {
        bb.has_square(BitBoard::from_square(*self))
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
    /// Gives back a bitboard of attack squares, a bitboard of checking rays, and whether or not
    /// the enemy king is attacked
    pub fn ray_old(&self, direction: &Direction, game: &Game) -> (BitBoard, BitBoard, bool) {
        let mut ray = EMPTY;
        let mut check_ray = EMPTY;
        let enemy = game.turn.opponent();
        let kingbb = game.get_pieces(&PieceType::King, &game.turn);

        let mut current = *self;
        let mut is_check = false;
        let mut is_check_ray = false;
        while let Some(forward) = current.walk(direction) {
            if let Some((piece, color)) = game.piece_lookup(forward) {
                let is_king = piece == PieceType::King;
                let is_enemy = color == enemy;
                if is_enemy {
                    ray.set(forward);
                    check_ray.set(forward);

                    if is_king {
                        is_check = true;
                        is_check_ray = true;
                    } else if let Some(extra) = forward.walk(direction) {
                        check_ray.set(extra);
                        let extrabb = BitBoard::from_square(extra);
                        is_check_ray = kingbb.has_square(extrabb);
                    }
                }

                if !(is_king && is_enemy) {
                    break;
                }
            } else {
                ray.set(forward);
                check_ray.set(forward);
            }

            current = forward;
        }

        if !is_check_ray {
            check_ray = EMPTY;
        }

        (ray, check_ray, is_check)
    }

    /// Generates moves for ray pieces. Also populates attack bitboards appropiately
    pub fn ray_moves(&self, directions: &[Direction], game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();

        for direction in directions {
            let (ray, _, _) = self.ray_old(direction, game);

            for sq in ray {
                let capture = game.piece_lookup(sq).map(|(piece, _)| piece);

                let m = Move::Normal {
                    from: *self,
                    to: sq,
                    capture,
                };

                moves.push(m);
            }
        }

        moves
    }

    /// Generates a ray of squares until either the end of the board, right before a friendly piece,
    /// or it ends right on an enemy piece. Used for ray pieces in move generation.
    pub fn ray(&self, direction: &Direction, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();
        let selfbb = BitBoard::from_square(*self);

        // Maybe I should take in color as a parameter?
        let enemy = game.determine_color(selfbb).unwrap_or(game.turn).opponent();

        let mut current = *self;
        let mut second_blocker = false;
        let mut is_check = false;
        while let Some(forward) = current.walk(direction) {
            let forwardbb = BitBoard::from_square(forward);
            moveinfo.check_rays |= forwardbb;

            if is_check {
                moveinfo.attacks |= forwardbb;
            }

            if let Some((piece, color)) = game.piece_lookup(forward) {
                let is_king = piece == PieceType::King;
                let is_enemy = color == enemy;

                if is_enemy && is_king {
                    is_check = true;
                }

                if second_blocker {
                    break;
                }

                moveinfo.attacks |= forwardbb;

                if is_enemy {
                    moveinfo.targets |= forwardbb;
                }

                second_blocker = true;
            } else if !second_blocker {
                moveinfo.targets |= forwardbb;
                moveinfo.attacks |= forwardbb;
            }

            current = forward;
        }

        if !is_check {
            moveinfo.check_rays = EMPTY;
        }

        moveinfo
    }

    /// Generates moveinfo for ray pieces
    pub fn rays(&self, directions: &[Direction], game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        for direction in directions {
            let raymoveinfo = self.ray(direction, game);

            moveinfo.targets |= raymoveinfo.targets;
            moveinfo.attacks |= raymoveinfo.attacks;
            moveinfo.check_rays |= raymoveinfo.check_rays;
        }

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::ALL_FILES, rank::ALL_RANKS};

    use super::*;

    #[test]
    fn in_bitboards() {
        let board = Game::default();
        let occupied = &board.occupied;

        let first = Square::C7;
        let second = Square::H1;
        let empty = Square::G3;

        assert!(first.in_bitboard(occupied));
        assert!(second.in_bitboard(occupied));
        assert!(!empty.in_bitboard(occupied));
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

    #[test]
    fn flip_side() {
        assert_eq!(Square::E4, Square::E5.flip_side());
        assert_eq!(Square::A1, Square::A8.flip_side());
        assert_eq!(Square::H1, Square::H8.flip_side());
        assert_eq!(Square::D2, Square::D7.flip_side());
        assert_eq!(Square::F6, Square::F3.flip_side());
        assert_eq!(Square::C3, Square::C6.flip_side());
        assert_eq!(Square::G7, Square::G2.flip_side());
        assert_eq!(Square::H3, Square::H3.flip_side().flip_side())
    }

    #[test]
    fn ray() {
        let fen = "r1bq1r1k/1p4pp/1pnp4/2p1pNb1/2B1P3/P1PP4/1P3PPP/R1BQ1RK1 b - - 0 14";
        let game = Game::from_fen(fen).unwrap();

        let rook = Square::F8;
        let direction = Direction::South;
        let mut expected = PieceMoveInfo::default();
        expected.targets.set(Square::F7);
        expected.targets.set(Square::F6);
        expected.targets.set(Square::F5);
        expected.attacks.set(Square::F7);
        expected.attacks.set(Square::F6);
        expected.attacks.set(Square::F5);

        let actual = rook.ray(&direction, &game);
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_rank_file() {
        for rank in ALL_RANKS {
            for file in ALL_FILES {
                let sq = Square::make_square(rank, file);
                assert_eq!(sq.get_rank(), rank);
                assert_eq!(sq.get_file(), file);
            }
        }
    }
}
