use crate::{
    movegen::pieces::piece::{PieceColor, PieceType},
    square::Square,
};

/// Bit packed table entry
struct PieceTableEntry(u16);

impl PieceTableEntry {
    fn new(square: Square, piece: Option<PieceType>, color: Option<PieceColor>) -> PieceTableEntry {
        let sq = square.to_int() as u16;

        let pc = match piece {
            Some(piece) => (piece.to_int() as u16) << 7, // bit 6 = 0, not None
            None => 1 << 6,                              // bit 6 = 1, None
        };

        let co = match color {
            Some(color) => (color.to_int() as u16) << 12, // bit 11 = 0, not None
            None => 1 << 11,                              // bit 11 = 1, None
        };

        Self(sq | pc | co)
    }

    /// Bits 1 to 6
    fn square(&self) -> Square {
        Square::new((self.0 & 0b111111) as u8)
    }

    /// Bit 7 -> None, bits 8 to 11 -> Some(PieceType)
    fn piece(&self) -> Option<PieceType> {
        if (self.0 >> 6) & 1 == 1 {
            return None;
        }
        PieceType::from_int(((self.0 >> 7) & 0b1111) as u8)
    }

    /// Bit 12 -> None, Bit 13 -> Some(PieceColor)
    fn color(&self) -> Option<PieceColor> {
        if (self.0 >> 11) & 1 == 1 {
            return None;
        }
        PieceColor::from_int(((self.0 >> 12) & 0b1) as u8)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn piece_table_entry() {
        let square = Square::H8;
        let piece = Some(PieceType::King);
        let color = Some(PieceColor::Black);
        let entry = PieceTableEntry::new(square, piece, color);
        assert_eq!(entry.square(), square);
        assert_eq!(entry.piece(), piece);
        assert_eq!(entry.color(), color);
    }
}
