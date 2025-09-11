use crate::board::PieceType;

impl PieceType {
    /// Gets the pieces value, for example, a pawn is 1.0. Does not consider turn.
    pub fn material_value(&self) -> f32 {
        match self {
            PieceType::Pawn => 1.0,
            PieceType::Knight => 3.0,
            PieceType::Bishop => 3.0,
            PieceType::Rook => 5.0,
            PieceType::Queen => 9.0,
            PieceType::King => 0.0,
        }
    }
}
