pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

macro_rules! color_field_getters {
    ($field_name:ident, $return_type:ty) => {
        paste::paste! {
            pub fn [<get_ $field_name _mut>](&mut self, color: &crate::movegen::pieces::piece::PieceColor) -> &mut $return_type {
                match color {
                    crate::movegen::pieces::piece::PieceColor::White => &mut self.[<white_ $field_name>],
                    crate::movegen::pieces::piece::PieceColor::Black => &mut self.[<black_ $field_name>],
                }
            }

            pub fn [<get_ $field_name>](&self, color: &crate::movegen::pieces::piece::PieceColor) -> &$return_type {
                match color {
                    crate::movegen::pieces::piece::PieceColor::White => &self.[<white_ $field_name>],
                    crate::movegen::pieces::piece::PieceColor::Black => &self.[<black_ $field_name>],
                }
            }
        }
    };
}
pub(crate) use color_field_getters;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    InProgress,
    Checkmate,
    Stalemate,
    Timeout,
    Repetition,
}
