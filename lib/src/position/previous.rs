use crate::{
    position::{castling::CastlingRights, game::State},
    square::Square,
};

/// Non-restoreable information needed to undo a move
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnRestoreable {
    pub castling_rights: CastlingRights,
    pub half_move_timeout: u8,
    pub en_passant_target: Option<Square>,
}

impl UnRestoreable {
    pub(crate) fn pack(self) -> PackedUnRestoreable {
        todo!();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PackedUnRestoreable(u32);

impl PackedUnRestoreable {
    pub(crate) fn unpack(self) -> UnRestoreable {
        todo!();
    }
}
