use std::fmt;

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

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct CastlingRights(u8);

impl fmt::Debug for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "CastlingRights({}) {{ white_kingside: {}, white_queenside: {}, black_kingside: {}, black_queenside: {} }}",
                self.to_int(),
                self.white_kingside(),
                self.white_queenside(),
                self.black_kingside(),
                self.black_queenside(),
            )
        } else {
            write!(f, "CastlingRights(\"{}\")", self.to_fen())
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self(
            Self::WHITE_QUEENSIDE
                | Self::WHITE_KINGSIDE
                | Self::BLACK_QUEENSIDE
                | Self::BLACK_KINGSIDE,
        )
    }
}

impl CastlingRights {
    const WHITE_QUEENSIDE: u8 = 0b0001;
    const WHITE_KINGSIDE: u8 = 0b0010;
    const BLACK_QUEENSIDE: u8 = 0b0100;
    const BLACK_KINGSIDE: u8 = 0b1000;

    pub fn empty() -> Self {
        Self(0)
    }

    pub(crate) const fn from_int(val: u8) -> CastlingRights {
        CastlingRights(val)
    }

    pub(crate) const fn to_int(self) -> u8 {
        self.0
    }

    pub fn from_fen(castling_fen: &str) -> Self {
        let mut out = 0;
        if castling_fen.contains('Q') {
            out |= Self::WHITE_QUEENSIDE;
        }
        if castling_fen.contains('K') {
            out |= Self::WHITE_KINGSIDE;
        }
        if castling_fen.contains('q') {
            out |= Self::BLACK_QUEENSIDE;
        }
        if castling_fen.contains('k') {
            out |= Self::BLACK_KINGSIDE;
        }
        Self(out)
    }

    pub fn to_fen(&self) -> String {
        let mut out = String::with_capacity(4);
        if self.white_kingside() {
            out.push('K');
        }
        if self.white_queenside() {
            out.push('Q');
        }
        if self.black_kingside() {
            out.push('k');
        }
        if self.black_queenside() {
            out.push('q');
        }
        if out.is_empty() {
            return '-'.to_string();
        }
        out
    }

    pub fn white_queenside(self) -> bool {
        self.0 & Self::WHITE_QUEENSIDE != 0
    }

    pub fn white_kingside(self) -> bool {
        self.0 & Self::WHITE_KINGSIDE != 0
    }

    pub fn black_queenside(self) -> bool {
        self.0 & Self::BLACK_QUEENSIDE != 0
    }

    pub fn black_kingside(self) -> bool {
        self.0 & Self::BLACK_KINGSIDE != 0
    }

    pub(crate) fn revoke_white(&mut self) {
        self.0 &= !(Self::WHITE_QUEENSIDE | Self::WHITE_KINGSIDE);
    }

    pub(crate) fn revoke_black(&mut self) {
        self.0 &= !(Self::BLACK_QUEENSIDE | Self::BLACK_KINGSIDE);
    }

    pub(crate) fn revoke_white_queenside(&mut self) {
        self.0 &= !Self::WHITE_QUEENSIDE;
    }

    pub(crate) fn revoke_white_kingside(&mut self) {
        self.0 &= !Self::WHITE_KINGSIDE;
    }

    pub(crate) fn revoke_black_queenside(&mut self) {
        self.0 &= !Self::BLACK_QUEENSIDE;
    }

    pub(crate) fn revoke_black_kingside(&mut self) {
        self.0 &= !Self::BLACK_KINGSIDE;
    }
}
