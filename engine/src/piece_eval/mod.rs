mod tables;

use whalecrab_lib::{
    movegen::pieces::piece::{PieceColor, PieceType},
    square::Square,
};

use crate::score::Score;

/// Gets the pieces value, for example, a pawn is 1.0. Does not consider turn.
pub const fn material_value(piece_type: PieceType) -> Score {
    let value = match piece_type {
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 1000,
    };

    Score::new(value)
}

/// Gets the positional value of a piece using a piece-square table
pub fn square_value(piece_type: PieceType, sq: Square, color: PieceColor, ratio: f64) -> Score {
    let index = match color {
        PieceColor::White => sq,
        PieceColor::Black => sq.flip_side(),
    }
    .to_int() as usize;

    let (midgame, endgame) = unsafe {
        match piece_type {
            PieceType::Pawn => (
                *tables::PAWN_MID.get_unchecked(index),
                *tables::PAWN_END.get_unchecked(index),
            ),
            PieceType::Knight => (
                *tables::KNIGHT_MID.get_unchecked(index),
                *tables::KNIGHT_END.get_unchecked(index),
            ),
            PieceType::Bishop => (
                *tables::BISHOP_MID.get_unchecked(index),
                *tables::BISHOP_END.get_unchecked(index),
            ),
            PieceType::Rook => (
                *tables::ROOK_MID.get_unchecked(index),
                *tables::ROOK_END.get_unchecked(index),
            ),
            PieceType::Queen => (
                *tables::QUEEN_MID.get_unchecked(index),
                *tables::QUEEN_END.get_unchecked(index),
            ),
            PieceType::King => (
                *tables::KING_MID.get_unchecked(index),
                *tables::KING_END.get_unchecked(index),
            ),
        }
    };

    let ratio_endgame = ratio;
    let ratio_midgame = 1.0 - ratio_endgame;
    let score = midgame as f64 * ratio_midgame + endgame as f64 * ratio_endgame;

    Score::new(score as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                square_value(piece, sq, PieceColor::White, 0.5),
                square_value(piece, sq.flip_side(), PieceColor::Black, 0.5),
                "Failed for {:?} at {:?}",
                piece,
                sq
            );
        }
    }
}
