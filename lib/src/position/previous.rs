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
    pub(crate) fn pack(self) -> PackedUnRestoreable {
        // Only the file is stored; rank is inferred from turn in unpack
        let en_passant_bits: u16 = match self.en_passant_target {
            Some(sq) => sq.get_file() as u16,
            None => 8,
        };

        PackedUnRestoreable(
            (self.castling_rights.to_int() as u16) << PackedUnRestoreable::CASTLING_OFFSET
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
pub(crate) struct PackedUnRestoreable(u16);

impl PackedUnRestoreable {
    const CASTLING_OFFSET: u16 = 0;
    const EN_PASSANT_OFFSET: u16 = 4;
    const HALF_MOVE_OFFSET: u16 = 8;

    pub(crate) fn unpack(self, turn: PieceColor) -> UnRestoreable {
        let castling_rights = CastlingRights::from_int((self.0 & 0xF) as u8);

        let en_passant_bits = (self.0 >> PackedUnRestoreable::EN_PASSANT_OFFSET) & 0xF;
        let en_passant_target = if en_passant_bits == 8 {
            None
        } else {
            let rank = match turn {
                PieceColor::White => Rank::Third,
                PieceColor::Black => Rank::Sixth,
            };
            let file = unsafe { File::from_int_unchecked(en_passant_bits as u8) };
            Some(Square::make_square(rank, file))
        };

        let half_move_timeout = (self.0 >> PackedUnRestoreable::HALF_MOVE_OFFSET) as u8;

        UnRestoreable {
            castling_rights,
            half_move_timeout,
            en_passant_target,
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
