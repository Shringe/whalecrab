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
