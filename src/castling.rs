use crate::{bitboard::BitBoard, board::Color, movegen::moves::{Move, MoveType}, square::Square};

pub const WHITE_TRAVERSES_CASTLING_QUEENSIDE: BitBoard =
    BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110);
pub const WHITE_TRAVERSES_CASTLING_KINGSIDE: BitBoard =
    BitBoard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000);
pub const BLACK_TRAVERSES_CASTLING_QUEENSIDE: BitBoard =
    BitBoard(0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub const BLACK_TRAVERSES_CASTLING_KINGSIDE: BitBoard =
    BitBoard(0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

pub const WHITE_CASTLES_QUEENSIDE: Move = Move {
    from: Square::E1,
    to: Square::C1,
    variant: MoveType::Castle(CastleSide::Queenside),
};

pub const WHITE_CASTLES_KINGSIDE: Move = Move {
    from: Square::E1,
    to: Square::G1,
    variant: MoveType::Castle(CastleSide::Kingside),
};

pub const BLACK_CASTLES_QUEENSIDE: Move = Move {
    from: Square::E8,
    to: Square::C8,
    variant: MoveType::Castle(CastleSide::Queenside),
};

pub const BLACK_CASTLES_KINGSIDE: Move = Move {
    from: Square::E8,
    to: Square::G8,
    variant: MoveType::Castle(CastleSide::Kingside),
};

#[derive(Debug, PartialEq)]
pub enum CastleSide {
    Queenside,
    Kingside,
}

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

    pub fn get_castle_rights(&self, color: &Color, side: &CastleSide) -> bool {
        match color {
            Color::White => match side {
                CastleSide::Queenside => self.white_queenside,
                CastleSide::Kingside => self.white_kingside,
            },
            Color::Black => match side {
                CastleSide::Queenside => self.black_queenside,
                CastleSide::Kingside => self.black_kingside,
            },
        }
    }

    pub fn revoke_castle_rights(&mut self, color: &Color, side: &CastleSide) {
        match color {
            Color::White => match side {
                CastleSide::Queenside => self.white_queenside = false,
                CastleSide::Kingside => self.white_kingside = false,
            },
            Color::Black => match side {
                CastleSide::Queenside => self.black_queenside = false,
                CastleSide::Kingside => self.black_kingside = false,
            },
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
