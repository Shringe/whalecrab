use crate::movegen::pieces::piece::{PieceColor, PieceType};

/// Bit-packed pair of squares.
/// Low nibble (bits 0–3): even square. High nibble (bits 4–7): odd square.
/// Nibble encoding:
///   1111 = None
///   0000 = white pawn   | 0110 = black pawn
///   1000 = white knight | 1110 = black knight
///   0100 = white bishop | 0001 = black bishop
///   1100 = white rook   | 1001 = black rook
///   0010 = white queen  | 0101 = black queen
///   1010 = white king   | 1101 = black king
#[derive(Clone)]
struct PieceTableEntry(u8);

impl PieceTableEntry {
    const EMPTY: u8 = 0b1111_1111;

    const fn encode(val: Option<(PieceType, PieceColor)>) -> u8 {
        match val {
            None => 0b1111,
            Some((PieceType::Pawn, PieceColor::White)) => 0b0000,
            Some((PieceType::Knight, PieceColor::White)) => 0b1000,
            Some((PieceType::Bishop, PieceColor::White)) => 0b0100,
            Some((PieceType::Rook, PieceColor::White)) => 0b1100,
            Some((PieceType::Queen, PieceColor::White)) => 0b0010,
            Some((PieceType::King, PieceColor::White)) => 0b1010,
            Some((PieceType::Pawn, PieceColor::Black)) => 0b0110,
            Some((PieceType::Knight, PieceColor::Black)) => 0b1110,
            Some((PieceType::Bishop, PieceColor::Black)) => 0b0001,
            Some((PieceType::Rook, PieceColor::Black)) => 0b1001,
            Some((PieceType::Queen, PieceColor::Black)) => 0b0101,
            Some((PieceType::King, PieceColor::Black)) => 0b1101,
        }
    }

    const fn decode(nibble: u8) -> Option<(PieceType, PieceColor)> {
        match nibble {
            0b1111 => None,
            0b0000 => Some((PieceType::Pawn, PieceColor::White)),
            0b1000 => Some((PieceType::Knight, PieceColor::White)),
            0b0100 => Some((PieceType::Bishop, PieceColor::White)),
            0b1100 => Some((PieceType::Rook, PieceColor::White)),
            0b0010 => Some((PieceType::Queen, PieceColor::White)),
            0b1010 => Some((PieceType::King, PieceColor::White)),
            0b0110 => Some((PieceType::Pawn, PieceColor::Black)),
            0b1110 => Some((PieceType::Knight, PieceColor::Black)),
            0b0001 => Some((PieceType::Bishop, PieceColor::Black)),
            0b1001 => Some((PieceType::Rook, PieceColor::Black)),
            0b0101 => Some((PieceType::Queen, PieceColor::Black)),
            0b1101 => Some((PieceType::King, PieceColor::Black)),
            _ => None,
        }
    }

    const fn first(&self) -> Option<(PieceType, PieceColor)> {
        Self::decode(self.0 & 0x0F)
    }

    const fn second(&self) -> Option<(PieceType, PieceColor)> {
        Self::decode((self.0 >> 4) & 0x0F)
    }

    const fn set_first(&mut self, val: Option<(PieceType, PieceColor)>) {
        self.0 = (self.0 & 0xF0) | Self::encode(val);
    }

    const fn set_second(&mut self, val: Option<(PieceType, PieceColor)>) {
        self.0 = (self.0 & 0x0F) | (Self::encode(val) << 4);
    }
}

#[derive(Clone)]
pub struct PieceTable([PieceTableEntry; 32]);

impl PieceTable {
    pub const fn new() -> Self {
        const EMPTY: PieceTableEntry = PieceTableEntry(PieceTableEntry::EMPTY);
        PieceTable([EMPTY; 32])
    }

    pub const fn get(&self, square: u8) -> Option<(PieceType, PieceColor)> {
        let entry = &self.0[(square >> 1) as usize];
        if square & 1 == 0 {
            entry.first()
        } else {
            entry.second()
        }
    }

    pub const fn set(&mut self, square: u8, val: Option<(PieceType, PieceColor)>) {
        let entry = &mut self.0[(square >> 1) as usize];
        if square & 1 == 0 {
            entry.set_first(val);
        } else {
            entry.set_second(val);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip_all_pieces() {
        let cases = [
            Some((PieceType::Pawn, PieceColor::White)),
            Some((PieceType::Knight, PieceColor::White)),
            Some((PieceType::Bishop, PieceColor::White)),
            Some((PieceType::Rook, PieceColor::White)),
            Some((PieceType::Queen, PieceColor::White)),
            Some((PieceType::King, PieceColor::White)),
            Some((PieceType::Pawn, PieceColor::Black)),
            Some((PieceType::Knight, PieceColor::Black)),
            Some((PieceType::Bishop, PieceColor::Black)),
            Some((PieceType::Rook, PieceColor::Black)),
            Some((PieceType::Queen, PieceColor::Black)),
            Some((PieceType::King, PieceColor::Black)),
            None,
        ];
        for val in cases {
            assert_eq!(PieceTableEntry::decode(PieceTableEntry::encode(val)), val);
        }
    }

    #[test]
    fn piece_table_set_get() {
        let mut table = PieceTable::new();
        table.set(0, Some((PieceType::Rook, PieceColor::White)));
        table.set(1, Some((PieceType::King, PieceColor::Black)));
        assert_eq!(table.get(0), Some((PieceType::Rook, PieceColor::White)));
        assert_eq!(table.get(1), Some((PieceType::King, PieceColor::Black)));
    }

    #[test]
    fn piece_table_clear() {
        let mut table = PieceTable::new();
        table.set(32, Some((PieceType::Queen, PieceColor::Black)));
        table.set(32, None);
        assert_eq!(table.get(32), None);
    }
}
