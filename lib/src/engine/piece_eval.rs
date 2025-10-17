use crate::{
    movegen::pieces::piece::{Color, PieceType},
    square::Square,
};

// Tables found from https://talkchess.com/viewtopic.php?t=76256
#[rustfmt::skip]
const KING_MID: [f32; 64] = [
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0, -0.05, -0.05, -0.05,  0.0,  0.0,
    0.0,  0.0,  0.1, -0.05, -0.05, -0.05,  0.1,  0.0
];

#[rustfmt::skip]
const QUEEN_MID: [f32; 64] = [
    -0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2,
    -0.1,  0.0,  0.0,  0.0,   0.0,  0.0,  0.0, -0.1,
    -0.1,  0.0,  0.05, 0.05,  0.05, 0.05, 0.0, -0.1,
    -0.05, 0.0,  0.05, 0.05,  0.05, 0.05, 0.0, -0.05,
    -0.05, 0.0,  0.05, 0.05,  0.05, 0.05, 0.0, -0.05,
    -0.1,  0.05, 0.05, 0.05,  0.05, 0.05, 0.0, -0.1,
    -0.1,  0.0,  0.05, 0.0,   0.0,  0.0,  0.0, -0.1,
    -0.2, -0.1, -0.1,  0.0,   0.0, -0.1, -0.1, -0.2
];

#[rustfmt::skip]
const ROOK_MID: [f32; 64] = [
    0.1,  0.1,  0.1,  0.1,  0.1,  0.1,  0.1,  0.1,
    0.1,  0.1,  0.1,  0.1,  0.1,  0.1,  0.1,  0.1,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.1,  0.1,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.1,  0.1,  0.05, 0.0,  0.0
];

#[rustfmt::skip]
const BISHOP_MID: [f32; 64] = [
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.0,  0.1,  0.0,  0.0,  0.0,  0.0,  0.1,  0.0,
    0.05, 0.0,  0.1,  0.0,  0.0,  0.1,  0.0,  0.05,
    0.0,  0.1,  0.0,  0.1,  0.1,  0.0,  0.1,  0.0,
    0.0,  0.1,  0.0,  0.1,  0.1,  0.0,  0.1,  0.0,
    0.0,  0.0, -0.1,  0.0,  0.0, -0.1,  0.0,  0.0
];

#[rustfmt::skip]
const KNIGHT_MID: [f32; 64] = [
    -0.05, -0.05, -0.05, -0.05, -0.05, -0.05, -0.05, -0.05,
    -0.05,  0.0,   0.0,   0.1,   0.1,   0.0,   0.0,  -0.05,
    -0.05,  0.05,  0.1,   0.1,   0.1,   0.1,   0.05, -0.05,
    -0.05,  0.05,  0.1,   0.15,  0.15,  0.1,   0.05, -0.05,
    -0.05,  0.05,  0.1,   0.15,  0.15,  0.1,   0.05, -0.05,
    -0.05,  0.05,  0.1,   0.1,   0.1,   0.1,   0.05, -0.05,
    -0.05,  0.0,   0.0,   0.05,  0.05,  0.0,   0.0,  -0.05,
    -0.05, -0.1,  -0.05, -0.05, -0.05, -0.05, -0.1,  -0.05
];

#[rustfmt::skip]
const PAWN_MID: [f32; 64] = [
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
    0.3,  0.3,  0.3,  0.4,  0.4,  0.3,  0.3,  0.3,
    0.2,  0.2,  0.2,  0.3,  0.3,  0.3,  0.2,  0.2,
    0.1,  0.1,  0.15, 0.25, 0.25, 0.15, 0.1,  0.1,
    0.05, 0.05, 0.05, 0.2,  0.2,  0.05, 0.05, 0.05,
    0.05, 0.0,  0.0,  0.05, 0.05, 0.0,  0.0,  0.05,
    0.05, 0.05, 0.05, -0.1, -0.1, 0.05, 0.05, 0.05,
    0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0
];

impl PieceType {
    /// Gets the pieces value, for example, a pawn is 1.0. Does not consider turn.
    pub fn material_value(&self) -> f32 {
        match self {
            PieceType::Pawn => 1.0,
            PieceType::Knight => 3.0,
            PieceType::Bishop => 3.0,
            PieceType::Rook => 5.0,
            PieceType::Queen => 9.0,
            PieceType::King => 10.0,
        }
    }

    /// Gets the positional value of a piece using a very basic piece-square table
    pub fn square_value(&self, sq: &Square, color: &Color) -> f32 {
        let table = match self {
            PieceType::Pawn => PAWN_MID,
            PieceType::Knight => KNIGHT_MID,
            PieceType::Bishop => BISHOP_MID,
            PieceType::Rook => ROOK_MID,
            PieceType::Queen => QUEEN_MID,
            PieceType::King => KING_MID,
        };

        // TODO. broken completely
        let idx = match color {
            Color::White => sq.to_int(),
            Color::Black => sq.flip_side().to_int(),
        };

        *table.get(idx as usize).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        movegen::pieces::piece::{Color, PieceType},
        square::Square,
    };

    #[test]
    fn balanced_square_value() {
        for (piece, sq) in [
            (PieceType::Pawn, Square::E4),
            (PieceType::Pawn, Square::D2),
            (PieceType::Pawn, Square::A7),
            (PieceType::Knight, Square::C3),
            (PieceType::Knight, Square::F6),
            (PieceType::Knight, Square::H1),
            (PieceType::Bishop, Square::D4),
            (PieceType::Bishop, Square::A1),
            (PieceType::Bishop, Square::G7),
            (PieceType::Rook, Square::A1),
            (PieceType::Rook, Square::E1),
            (PieceType::Rook, Square::H8),
            (PieceType::Queen, Square::D1),
            (PieceType::Queen, Square::E5),
            (PieceType::Queen, Square::B6),
            (PieceType::King, Square::E1),
            (PieceType::King, Square::G1),
            (PieceType::King, Square::D4),
        ] {
            assert_eq!(
                piece.square_value(&sq, &Color::White),
                piece.square_value(&sq.flip_side(), &Color::Black),
                "Failed for {:?} at {:?}",
                piece,
                sq
            );
        }
    }
}
