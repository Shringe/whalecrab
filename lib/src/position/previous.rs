use crate::{
    file::File, movegen::pieces::piece::PieceColor, position::castling::CastlingRights, rank::Rank,
    square::Square,
};

/// Non-restoreable information needed to undo a move.
/// You can pack this type by calling UnRestoreable::pack().
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct UnRestoreable {
    pub(crate) castling_rights: CastlingRights,
    pub(crate) en_passant_target: Option<Square>,
    pub(crate) half_move_timeout: u8,
}

impl UnRestoreable {
    fn pack(self) -> PackedUnRestoreable {
        // Only the file is stored; rank is inferred from turn in unpack
        let en_passant_bits: u16 = match self.en_passant_target {
            Some(sq) => sq.get_file() as u16,
            None => PackedUnRestoreable::EN_PASSANT_SENTINEL,
        };

        PackedUnRestoreable(
            (self.castling_rights.to_int() as u16)
                | en_passant_bits << PackedUnRestoreable::EN_PASSANT_OFFSET
                | (self.half_move_timeout as u16) << PackedUnRestoreable::HALF_MOVE_OFFSET,
        )
    }
}

// TODO: do we really need self.half_move_timeout?
/// Bit packed UnRestoreable. Call PackedUnRestoreable::unpack() to get back the UnRestoreable.
/// Bit layout (16 bits total):
/// [0..3]  castling_rights   (4 bits)
/// [4..7]  en_passant_target (4 bits, 0-7 = File, 8 = None)
/// [8..15] half_move_timeout (8 bits)
#[derive(Clone, Copy, PartialEq, Debug)]
struct PackedUnRestoreable(u16);

impl PackedUnRestoreable {
    const CASTLING_MASK: u16 = 0xF;
    const EN_PASSANT_OFFSET: u16 = 4;
    const EN_PASSANT_MASK: u16 = 0xF;
    const EN_PASSANT_SENTINEL: u16 = 8;
    const HALF_MOVE_OFFSET: u16 = 8;

    /// This is not meant to be unpacked.
    /// If ever unpacked this will default to zeros because of en_passant_bits branching.
    #[allow(clippy::unusual_byte_groupings)]
    const UN_INITIALIZED: PackedUnRestoreable = PackedUnRestoreable(0b0000_1111_00000000);

    fn unpack(self, turn: PieceColor) -> UnRestoreable {
        let castling_rights =
            CastlingRights::from_int((self.0 & PackedUnRestoreable::CASTLING_MASK) as u8);
        let half_move_timeout = (self.0 >> PackedUnRestoreable::HALF_MOVE_OFFSET) as u8;

        let en_passant_bits = (self.0 >> PackedUnRestoreable::EN_PASSANT_OFFSET)
            & PackedUnRestoreable::EN_PASSANT_MASK;

        let en_passant_target = if en_passant_bits < PackedUnRestoreable::EN_PASSANT_SENTINEL {
            let rank = match turn {
                PieceColor::White => Rank::Third,
                PieceColor::Black => Rank::Sixth,
            };

            // # Safety: en_passant_bits is checked to be below 8 above
            let file = unsafe { File::from_int_unchecked(en_passant_bits as u8) };
            Some(Square::make_square(rank, file))
        } else {
            None
        };

        UnRestoreable {
            castling_rights,
            half_move_timeout,
            en_passant_target,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PositionHistory {
    history: [PackedUnRestoreable; PositionHistory::MAX_SIZE as usize],
    len: u8,
}

impl PositionHistory {
    const MAX_SIZE: u8 = u8::MAX;

    pub(crate) const fn new() -> PositionHistory {
        PositionHistory {
            history: [PackedUnRestoreable::UN_INITIALIZED; PositionHistory::MAX_SIZE as usize],
            len: 0,
        }
    }

    /// Packs and stores `unrestoreable` for the given `turn`
    pub(crate) fn push(&mut self, unrestoreable: UnRestoreable) {
        #[cfg(debug_assertions)]
        {
            self.history[self.len as usize] = unrestoreable.pack();
        }

        #[cfg(not(debug_assertions))]
        {
            unsafe { *self.history.get_unchecked_mut(self.len as usize) = unrestoreable.pack() };
        }

        self.len += 1;
    }

    /// Pops and unpacks the last stored position for the given `turn`.
    /// `turn` must be the active player at the time the position was pushed.
    pub(crate) fn pop(&mut self, turn: PieceColor) -> Option<UnRestoreable> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        #[cfg(debug_assertions)]
        {
            let out = std::mem::replace(
                &mut self.history[self.len as usize],
                PackedUnRestoreable::UN_INITIALIZED,
            );
            assert_ne!(
                out,
                PackedUnRestoreable::UN_INITIALIZED,
                "Tried to restore an uninitialized or invalid position!"
            );
            Some(out.unpack(turn))
        }

        #[cfg(not(debug_assertions))]
        {
            let out = unsafe { self.history.get_unchecked(self.len as usize) };
            Some(out.unpack(turn))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn roundtrip(original: UnRestoreable, turn: PieceColor) {
        let unpacked = original.pack().unpack(turn);
        assert_eq!(original, unpacked);
    }

    #[test]
    fn no_en_passant() {
        roundtrip(
            UnRestoreable {
                castling_rights: CastlingRights::from_int(0b1111),
                half_move_timeout: 0,
                en_passant_target: None,
            },
            PieceColor::White,
        );
    }

    #[test]
    fn en_passant_white() {
        roundtrip(
            UnRestoreable {
                castling_rights: CastlingRights::from_int(0b1010),
                half_move_timeout: 10,
                en_passant_target: Some(Square::make_square(Rank::Third, File::E)),
            },
            PieceColor::White,
        );
    }

    #[test]
    fn en_passant_black() {
        roundtrip(
            UnRestoreable {
                castling_rights: CastlingRights::from_int(0b0101),
                half_move_timeout: 25,
                en_passant_target: Some(Square::make_square(Rank::Sixth, File::D)),
            },
            PieceColor::Black,
        );
    }

    #[test]
    fn no_castling_rights() {
        roundtrip(
            UnRestoreable {
                castling_rights: CastlingRights::from_int(0b0000),
                half_move_timeout: 49,
                en_passant_target: None,
            },
            PieceColor::Black,
        );
    }

    #[test]
    fn max_half_move_timeout() {
        roundtrip(
            UnRestoreable {
                castling_rights: CastlingRights::from_int(0b1111),
                half_move_timeout: u8::MAX,
                en_passant_target: None,
            },
            PieceColor::White,
        );
    }
}
