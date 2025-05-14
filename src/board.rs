use crate::{
    bitboard::{BitBoard, EMPTY},
    square::Square,
};

#[derive(Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub struct Board {
    pub white_pawn_bitboard: BitBoard,
    pub white_knight_bitboard: BitBoard,
    pub white_bishop_bitboard: BitBoard,
    pub white_rook_bitboard: BitBoard,
    pub white_queen_bitboard: BitBoard,
    pub white_king_bitboard: BitBoard,

    pub black_pawn_bitboard: BitBoard,
    pub black_knight_bitboard: BitBoard,
    pub black_bishop_bitboard: BitBoard,
    pub black_rook_bitboard: BitBoard,
    pub black_queen_bitboard: BitBoard,
    pub black_king_bitboard: BitBoard,

    pub is_whites_turn: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            white_pawn_bitboard: BitBoard::INITIAL_WHITE_PAWN,
            white_knight_bitboard: BitBoard::INITIAL_WHITE_KNIGHT,
            white_bishop_bitboard: BitBoard::INITIAL_WHITE_BISHOP,
            white_rook_bitboard: BitBoard::INITIAL_WHITE_ROOK,
            white_queen_bitboard: BitBoard::INITIAL_WHITE_QUEEN,
            white_king_bitboard: BitBoard::INITIAL_WHITE_KING,

            black_pawn_bitboard: BitBoard::INITIAL_BLACK_PAWN,
            black_knight_bitboard: BitBoard::INITIAL_BLACK_KNIGHT,
            black_bishop_bitboard: BitBoard::INITIAL_BLACK_BISHOP,
            black_rook_bitboard: BitBoard::INITIAL_BLACK_ROOK,
            black_queen_bitboard: BitBoard::INITIAL_BLACK_QUEEN,
            black_king_bitboard: BitBoard::INITIAL_BLACK_KING,

            is_whites_turn: true,
        }
    }

    pub fn occupied_white_bitboard(&self) -> BitBoard {
        self.white_pawn_bitboard
            | self.white_knight_bitboard
            | self.white_bishop_bitboard
            | self.white_rook_bitboard
            | self.white_queen_bitboard
            | self.white_king_bitboard
    }

    pub fn occupied_black_bitboard(&self) -> BitBoard {
        self.black_pawn_bitboard
            | self.black_knight_bitboard
            | self.black_bishop_bitboard
            | self.black_rook_bitboard
            | self.black_queen_bitboard
            | self.black_king_bitboard
    }

    pub fn occupied_bitboard(&self) -> BitBoard {
        self.occupied_white_bitboard() | self.occupied_black_bitboard()
    }

    /// Determines color of standing piece
    pub fn determine_color(&self, sq: Square) -> Option<Color> {
        let pos = BitBoard::from_square(sq);
        if pos & self.occupied_white_bitboard() != EMPTY {
            Some(Color::White)
        } else if pos & self.occupied_black_bitboard() != EMPTY {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Determines type of standing piece
    pub fn determine_piece(&self, sq: Square) -> Option<Piece> {
        let pos = BitBoard::from_square(sq.clone());
        if pos & self.occupied_bitboard() == EMPTY {
            return None;
        }

        if pos & (self.white_pawn_bitboard | self.black_pawn_bitboard) != EMPTY {
            Some(Piece::Pawn)
        } else if pos & (self.white_knight_bitboard | self.black_knight_bitboard) != EMPTY {
            Some(Piece::Knight)
        } else if pos & (self.white_bishop_bitboard | self.black_bishop_bitboard) != EMPTY {
            Some(Piece::Bishop)
        } else if pos & (self.white_rook_bitboard | self.black_rook_bitboard) != EMPTY {
            Some(Piece::Rook)
        } else if pos & (self.white_queen_bitboard | self.black_queen_bitboard) != EMPTY {
            Some(Piece::Queen)
        } else if pos & (self.white_king_bitboard | self.black_king_bitboard) != EMPTY {
            Some(Piece::King)
        } else {
            panic!("Can't determine piece type of square {:?}!", sq);
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_colors() {
        let board = Board::default();

        let white = Square::C2;
        let empty = Square::G4;
        let black = Square::B8;

        assert_eq!(board.determine_color(white), Some(Color::White));
        assert_eq!(board.determine_color(empty), None);
        assert_eq!(board.determine_color(black), Some(Color::Black));
    }

    #[test]
    fn determine_pieces() {
        let board = Board::default();

        let pawn = Square::C2;
        let empty = Square::G4;
        let knight = Square::B8;

        assert_eq!(board.determine_piece(pawn), Some(Piece::Pawn));
        assert_eq!(board.determine_piece(empty), None);
        assert_eq!(board.determine_piece(knight), Some(Piece::Knight));
    }
}
