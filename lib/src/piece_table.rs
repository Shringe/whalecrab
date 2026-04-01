use crate::movegen::pieces::piece::{PieceColor, PieceType};

/// Bit packed table entry.
/// Layout (0-indexed bits):
/// - Bits 0-3: Option<PieceType> (4 bits; sentinel 0b1111 = None)
/// - Bits 4-5: Option<PieceColor> (2 bits; sentinel 0b11 = None)
struct PieceTableEntry(u8);

impl PieceTableEntry {
    const PIECE_SENTINAL: u8 = 0b1111;
    const COLOR_SENTINAL: u8 = 0b11;

    const fn new(piece: Option<PieceType>, color: Option<PieceColor>) -> PieceTableEntry {
        let mut out = match piece {
            Some(p) => p.to_int(),
            None => Self::PIECE_SENTINAL,
        };

        out |= match color {
            Some(c) => c.to_int() << 4,
            None => Self::COLOR_SENTINAL << 4,
        };

        PieceTableEntry(out)
    }

    /// Bits 0-3; sentinel 0b1111 = None
    const fn piece(&self) -> Option<PieceType> {
        PieceType::from_int(self.0 & Self::PIECE_SENTINAL)
    }

    /// Bits 4-5; sentinel 0b11 = None
    const fn color(&self) -> Option<PieceColor> {
        PieceColor::from_int((self.0 >> 4) & Self::COLOR_SENTINAL)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn piece_table_entry() {
        let piece = Some(PieceType::King);
        let color = Some(PieceColor::Black);
        let entry = PieceTableEntry::new(piece, color);
        assert_eq!(entry.piece(), piece);
        assert_eq!(entry.color(), color);
    }

    #[test]
    fn empty_piece_table_entry() {
        let piece = None;
        let color = None;
        let entry = PieceTableEntry::new(piece, color);
        assert_eq!(entry.piece(), piece);
        assert_eq!(entry.color(), color);
    }
}
