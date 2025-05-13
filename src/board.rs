use crate::bitboard::BitBoard;

#[derive(Debug)]
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
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
