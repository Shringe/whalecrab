use crate::{bitboard::BitBoard, square::Square};

pub const BLACK_CASTLE_KINGSIDE_NEEDS_CLEAR: BitBoard =
    BitBoard::new(0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub const BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR: BitBoard =
    BitBoard::new(0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub const WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000);
pub const WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110);

pub const BLACK_CASTLE_KINGSIDE_KING_FROM: Square = Square::E8;
pub const BLACK_CASTLE_QUEENSIDE_KING_FROM: Square = Square::E8;
pub const WHITE_CASTLE_KINGSIDE_KING_FROM: Square = Square::E1;
pub const WHITE_CASTLE_QUEENSIDE_KING_FROM: Square = Square::E1;

pub const BLACK_CASTLE_KINGSIDE_KING_TO: Square = Square::G8;
pub const BLACK_CASTLE_QUEENSIDE_KING_TO: Square = Square::C8;
pub const WHITE_CASTLE_KINGSIDE_KING_TO: Square = Square::G1;
pub const WHITE_CASTLE_QUEENSIDE_KING_TO: Square = Square::C1;

pub const BLACK_CASTLE_KINGSIDE_KING_FROM_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_KINGSIDE_KING_FROM);
pub const BLACK_CASTLE_QUEENSIDE_KING_FROM_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_QUEENSIDE_KING_FROM);
pub const WHITE_CASTLE_KINGSIDE_KING_FROM_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_KINGSIDE_KING_FROM);
pub const WHITE_CASTLE_QUEENSIDE_KING_FROM_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_QUEENSIDE_KING_FROM);

pub const BLACK_CASTLE_KINGSIDE_KING_TO_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_KINGSIDE_KING_TO);
pub const BLACK_CASTLE_QUEENSIDE_KING_TO_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_QUEENSIDE_KING_TO);
pub const WHITE_CASTLE_KINGSIDE_KING_TO_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_KINGSIDE_KING_TO);
pub const WHITE_CASTLE_QUEENSIDE_KING_TO_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_QUEENSIDE_KING_TO);

pub const BLACK_CASTLE_KINGSIDE_ROOK_FROM: Square = Square::H8;
pub const BLACK_CASTLE_QUEENSIDE_ROOK_FROM: Square = Square::A8;
pub const WHITE_CASTLE_KINGSIDE_ROOK_FROM: Square = Square::H1;
pub const WHITE_CASTLE_QUEENSIDE_ROOK_FROM: Square = Square::A1;

pub const BLACK_CASTLE_KINGSIDE_ROOK_TO: Square = Square::F8;
pub const BLACK_CASTLE_QUEENSIDE_ROOK_TO: Square = Square::D8;
pub const WHITE_CASTLE_KINGSIDE_ROOK_TO: Square = Square::F1;
pub const WHITE_CASTLE_QUEENSIDE_ROOK_TO: Square = Square::D1;

pub const BLACK_CASTLE_KINGSIDE_ROOK_FROM_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_KINGSIDE_ROOK_FROM);
pub const BLACK_CASTLE_QUEENSIDE_ROOK_FROM_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_QUEENSIDE_ROOK_FROM);
pub const WHITE_CASTLE_KINGSIDE_ROOK_FROM_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_KINGSIDE_ROOK_FROM);
pub const WHITE_CASTLE_QUEENSIDE_ROOK_FROM_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_QUEENSIDE_ROOK_FROM);

pub const BLACK_CASTLE_KINGSIDE_ROOK_TO_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_KINGSIDE_ROOK_TO);
pub const BLACK_CASTLE_QUEENSIDE_ROOK_TO_BB: BitBoard =
    BitBoard::from_square(BLACK_CASTLE_QUEENSIDE_ROOK_TO);
pub const WHITE_CASTLE_KINGSIDE_ROOK_TO_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_KINGSIDE_ROOK_TO);
pub const WHITE_CASTLE_QUEENSIDE_ROOK_TO_BB: BitBoard =
    BitBoard::from_square(WHITE_CASTLE_QUEENSIDE_ROOK_TO);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CastleSide {
    Queenside,
    Kingside,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct CastlingRights {
    white_queenside: bool,
    white_kingside: bool,
    black_queenside: bool,
    black_kingside: bool,
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

impl CastlingRights {
    pub fn empty() -> Self {
        Self {
            white_queenside: false,
            white_kingside: false,
            black_queenside: false,
            black_kingside: false,
        }
    }

    pub fn from_fen(castling_fen: &str) -> Self {
        Self {
            white_queenside: castling_fen.contains('Q'),
            white_kingside: castling_fen.contains('K'),
            black_queenside: castling_fen.contains('q'),
            black_kingside: castling_fen.contains('k'),
        }
    }

    pub fn white_queenside(&self) -> bool {
        self.white_queenside
    }

    pub fn white_kingside(&self) -> bool {
        self.white_kingside
    }

    pub fn black_queenside(&self) -> bool {
        self.black_queenside
    }

    pub fn black_kingside(&self) -> bool {
        self.black_kingside
    }

    pub(crate) fn revoke_white(&mut self) {
        self.revoke_white_queenside();
        self.revoke_white_kingside();
    }

    pub(crate) fn revoke_black(&mut self) {
        self.revoke_black_queenside();
        self.revoke_black_kingside();
    }

    pub(crate) fn revoke_white_queenside(&mut self) {
        self.white_queenside = false;
    }

    pub(crate) fn revoke_white_kingside(&mut self) {
        self.white_kingside = false;
    }

    pub(crate) fn revoke_black_queenside(&mut self) {
        self.black_queenside = false;
    }

    pub(crate) fn revoke_black_kingside(&mut self) {
        self.black_kingside = false;
    }
}
