use crate::bitboard::BitBoard;

pub const WHITE_TRAVERSES_CASTLING_QUEENSIDE: BitBoard =
    BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110);
pub const WHITE_TRAVERSES_CASTLING_KINGSIDE: BitBoard =
    BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000);
pub const BLACK_TRAVERSES_CASTLING_QUEENSIDE: BitBoard =
    BitBoard(0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub const BLACK_TRAVERSES_CASTLING_KINGSIDE: BitBoard =
    BitBoard(0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

#[derive(Debug, Clone, PartialEq)]
pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

impl CastlingRights {
    pub fn empty() -> Self {
        Self {
            white_queenside: false,
            white_kingside: false,
            black_queenside: false,
            black_kingside: false,
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_queenside: true,
            white_kingside: true,
            black_queenside: true,
            black_kingside: true,
        }
    }
}
