use crate::{
    engine::score::Score,
    movegen::pieces::piece::{Color, PieceType},
    square::Square,
};

// Tables found from https://talkchess.com/viewtopic.php?t=76256
// Temporary i32 tables (only exist at compile time, not in binary)
#[rustfmt::skip]
const KING_MID: [i32; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,  -5,  -5,  -5,   0,   0,
    0,   0,  10,  -5,  -5,  -5,  10,   0
];

#[rustfmt::skip]
const QUEEN_MID: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
     -5,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,   0,   0, -10, -10, -20
];

#[rustfmt::skip]
const ROOK_MID: [i32; 64] = [
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,  10,  10,   0,   0,   0,
     0,   0,   0,  10,  10,   5,   0,   0
];

#[rustfmt::skip]
const BISHOP_MID: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,  10,   0,   0,   0,   0,  10,   0,
     5,   0,  10,   0,   0,  10,   0,   5,
     0,  10,   0,  10,  10,   0,  10,   0,
     0,  10,   0,  10,  10,   0,  10,   0,
     0,   0, -10,   0,   0, -10,   0,   0
];

#[rustfmt::skip]
const KNIGHT_MID: [i32; 64] = [
    -5,  -5,  -5,  -5,  -5,  -5,  -5,  -5,
    -5,   0,   0,  10,  10,   0,   0,  -5,
    -5,   5,  10,  10,  10,  10,   5,  -5,
    -5,   5,  10,  15,  15,  10,   5,  -5,
    -5,   5,  10,  15,  15,  10,   5,  -5,
    -5,   5,  10,  10,  10,  10,   5,  -5,
    -5,   0,   0,   5,   5,   0,   0,  -5,
    -5, -10,  -5,  -5,  -5,  -5, -10,  -5
];

#[rustfmt::skip]
const PAWN_MID: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
    30,  30,  30,  40,  40,  30,  30,  30,
    20,  20,  20,  30,  30,  30,  20,  20,
    10,  10,  15,  25,  25,  15,  10,  10,
     5,   5,   5,  20,  20,   5,   5,   5,
     5,   0,   0,   5,   5,   0,   0,   5,
     5,   5,   5, -10, -10,   5,   5,   5,
     0,   0,   0,   0,   0,   0,   0,   0
];

impl PieceType {
    /// Gets the pieces value, for example, a pawn is 1.0. Does not consider turn.
    pub fn material_value(&self) -> Score {
        let value = match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 1000,
        };

        Score::new(value)
    }

    /// Gets the positional value of a piece using a very basic piece-square table
    pub fn square_value(&self, sq: &Square, color: &Color) -> Score {
        let table = match self {
            PieceType::Pawn => PAWN_MID,
            PieceType::Knight => KNIGHT_MID,
            PieceType::Bishop => BISHOP_MID,
            PieceType::Rook => ROOK_MID,
            PieceType::Queen => QUEEN_MID,
            PieceType::King => KING_MID,
        };

        let idx = match color {
            Color::White => sq.to_int(),
            Color::Black => sq.flip_side().to_int(),
        };

        let value = *table.get(idx as usize).unwrap();
        Score::new(value)
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
