use crate::{
    bitboard::{BitBoard, EMPTY},
    square::Square,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent(&self) -> Color {
        match &self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone)]
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

    pub en_passant_target: Option<Square>,

    pub turn: Color,
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

            en_passant_target: None,

            turn: Color::White,
        }
    }

    pub fn set_occupied_bitboard(&mut self, piece: &PieceType, color: &Color, new: BitBoard) {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawn_bitboard = new,
                PieceType::Knight => self.white_knight_bitboard = new,
                PieceType::Bishop => self.white_bishop_bitboard = new,
                PieceType::Rook => self.white_rook_bitboard = new,
                PieceType::Queen => self.white_queen_bitboard = new,
                PieceType::King => self.white_king_bitboard = new,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawn_bitboard = new,
                PieceType::Knight => self.black_knight_bitboard = new,
                PieceType::Bishop => self.black_bishop_bitboard = new,
                PieceType::Rook => self.black_rook_bitboard = new,
                PieceType::Queen => self.black_queen_bitboard = new,
                PieceType::King => self.black_king_bitboard = new,
            },
        }
    }

    pub fn get_occupied_bitboard(&self, piece: &PieceType, color: &Color) -> BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawn_bitboard,
                PieceType::Knight => self.white_knight_bitboard,
                PieceType::Bishop => self.white_bishop_bitboard,
                PieceType::Rook => self.white_rook_bitboard,
                PieceType::Queen => self.white_queen_bitboard,
                PieceType::King => self.white_king_bitboard,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawn_bitboard,
                PieceType::Knight => self.black_knight_bitboard,
                PieceType::Bishop => self.black_bishop_bitboard,
                PieceType::Rook => self.black_rook_bitboard,
                PieceType::Queen => self.black_queen_bitboard,
                PieceType::King => self.black_king_bitboard,
            },
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
    pub fn determine_piece(&self, sq: Square) -> Option<PieceType> {
        let pos = BitBoard::from_square(sq);
        if pos & self.occupied_bitboard() == EMPTY {
            return None;
        }

        if pos & (self.white_pawn_bitboard | self.black_pawn_bitboard) != EMPTY {
            Some(PieceType::Pawn)
        } else if pos & (self.white_knight_bitboard | self.black_knight_bitboard) != EMPTY {
            Some(PieceType::Knight)
        } else if pos & (self.white_bishop_bitboard | self.black_bishop_bitboard) != EMPTY {
            Some(PieceType::Bishop)
        } else if pos & (self.white_rook_bitboard | self.black_rook_bitboard) != EMPTY {
            Some(PieceType::Rook)
        } else if pos & (self.white_queen_bitboard | self.black_queen_bitboard) != EMPTY {
            Some(PieceType::Queen)
        } else if pos & (self.white_king_bitboard | self.black_king_bitboard) != EMPTY {
            Some(PieceType::King)
        } else {
            panic!("Can't determine piece type of square {:?}!", sq);
        }
    }

    /// Switches the players' turn and removes en_passant_target
    pub fn next_turn(&mut self) {
        self.turn = self.turn.opponent();
        self.en_passant_target = None;
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
        let queen = Square::D1;

        assert_eq!(board.determine_color(white), Some(Color::White));
        assert_eq!(board.determine_color(empty), None);
        assert_eq!(board.determine_color(black), Some(Color::Black));
        assert_eq!(board.determine_color(queen), Some(Color::White));
    }

    #[test]
    fn determine_pieces() {
        let board = Board::default();

        let pawn = Square::C2;
        let empty = Square::G4;
        let knight = Square::B8;
        let queen = Square::D8;

        assert_eq!(board.determine_piece(pawn), Some(PieceType::Pawn));
        assert_eq!(board.determine_piece(empty), None);
        assert_eq!(board.determine_piece(knight), Some(PieceType::Knight));
        assert_eq!(board.determine_piece(queen), Some(PieceType::Queen));
    }

    #[test]
    fn get_occupied_bitboards() {
        let board = Board::default();

        let white_pawns = board.get_occupied_bitboard(&PieceType::Pawn, &Color::White);
        assert_eq!(white_pawns, board.white_pawn_bitboard);
        assert!(BitBoard::from_square(Square::A2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::H2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::A3) & white_pawns == EMPTY);
        assert!(BitBoard::from_square(Square::E4) & white_pawns == EMPTY);

        let black_rooks = board.get_occupied_bitboard(&PieceType::Rook, &Color::Black);
        assert_eq!(black_rooks, board.black_rook_bitboard);
        assert!(BitBoard::from_square(Square::A8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::H8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::B7) & black_rooks == EMPTY);
        assert!(BitBoard::from_square(Square::E5) & black_rooks == EMPTY);
    }
}
