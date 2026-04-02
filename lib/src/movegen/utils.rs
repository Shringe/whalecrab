// These are unsafe for now because self.get_pieces_mut borrows the entirety of &mut self instead
// of only the &mut pieces it returns. This can probably be avoided by providing a macro that does
// the same thing as self.get_pieces_mut in the future.
#[macro_export]
macro_rules! remove_piece {
    ($game:expr, $pieces:expr, $sqbb:expr, $sq:expr) => {
        $game.piece_table.set($sq, None);
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            *$pieces ^= $sqbb;
        }
    };
}

#[macro_export]
macro_rules! add_piece {
    ($game:expr, $pieces:expr, $sqbb:expr, $sq:expr, $piece:expr, $color:expr) => {
        $game.piece_table.set($sq, Some(($piece, $color)));
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            *$pieces |= $sqbb;
        }
    };
}

#[macro_export]
macro_rules! castle {
    ($game:expr, $kings:expr, $rooks:expr,
     $king_from_bb:expr, $king_from:expr, $king_to_bb:expr, $king_to:expr,
     $rook_from_bb:expr, $rook_from:expr, $rook_to_bb:expr, $rook_to:expr,
     $color:expr) => {{
        remove_piece!($game, $kings, $king_from_bb, $king_from);
        add_piece!(
            $game,
            $kings,
            $king_to_bb,
            $king_to,
            PieceType::King,
            $color
        );
        remove_piece!($game, $rooks, $rook_from_bb, $rook_from);
        add_piece!(
            $game,
            $rooks,
            $rook_to_bb,
            $rook_to,
            PieceType::Rook,
            $color
        );
    }};
}
