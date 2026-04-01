use crate::{
    movegen::pieces::piece::{PieceColor, PieceType},
    square::Square,
};

/// Bit packed table entry.
/// Layout (0-indexed bits):
/// - Bits  0- 5: Square (6 bits, values 0-63)
/// - Bits  6- 9: Option<PieceType> (4 bits; sentinel 0b1111 = None)
/// - Bits 10-11: Option<PieceColor> (2 bits; sentinel 0b11 = None)
struct PieceTableEntry(u16);

impl PieceTableEntry {
    const PIECE_SENTINAL: u16 = 0b1111;
    const COLOR_SENTINAL: u16 = 0b11;

    fn new(square: Square, piece: Option<PieceType>, color: Option<PieceColor>) -> PieceTableEntry {
        let sq = square.to_int() as u16;
        let pc = match piece {
            Some(p) => (p.to_int() as u16) << 6,
            None => Self::PIECE_SENTINAL << 6,
        };
        let co = match color {
            Some(c) => (c.to_int() as u16) << 10,
            None => Self::COLOR_SENTINAL << 10,
        };
        PieceTableEntry(sq | pc | co)
    }

    /// Bits 0-5
    fn square(&self) -> Square {
        Square::new((self.0 & 0b11_1111) as u8)
    }

    /// Bits 6-9; sentinel 0b1111 = None
    fn piece(&self) -> Option<PieceType> {
        PieceType::from_int(((self.0 >> 6) & 0b1111) as u8)
    }

    /// Bits 10-11; sentinel 0b11 = None
    fn color(&self) -> Option<PieceColor> {
        PieceColor::from_int(((self.0 >> 10) & 0b11) as u8)
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

    #[test]
    fn empty_piece_table_entry() {
        let square = Square::H8;
        let piece = None;
        let color = None;
        let entry = PieceTableEntry::new(square, piece, color);
        assert_eq!(entry.square(), square);
        assert_eq!(entry.piece(), piece);
        assert_eq!(entry.color(), color);
    }
}
