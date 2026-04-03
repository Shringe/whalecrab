use crate::{
    position::{castling::CastlingRights, game::State},
    square::Square,
};

/// Non-restoreable information needed to undo a move
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnRestoreable {
    pub castling_rights: CastlingRights,
    pub half_move_timeout: usize,
    pub en_passant_target: Option<Square>,
    // Not technically necessary but probably much faster to remember
    pub state: State,
}
