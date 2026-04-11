/// During move making you should always remove the previous pieces before adding new ones so that their mailboxes don't collide
#[macro_export]
macro_rules! remove_piece {
    ($game:expr, $pieces:expr, $sqbb:expr, $sq:expr) => {
        $game.piece_table.set($sq, None);
        *$pieces ^= $sqbb;
    };
}

/// During move making you should always remove the previous pieces before adding new ones so that their mailboxes don't collide
#[macro_export]
macro_rules! add_piece {
    ($game:expr, $pieces:expr, $sqbb:expr, $sq:expr, $piece:expr, $color:expr) => {
        $game.piece_table.set($sq, Some(($piece, $color)));
        *$pieces |= $sqbb;
    };
}

#[macro_export]
macro_rules! castle {
    ($game:expr, $kings:expr, $rooks:expr,
     $king_from_bb:expr, $king_from:expr, $king_to_bb:expr, $king_to:expr,
     $rook_from_bb:expr, $rook_from:expr, $rook_to_bb:expr, $rook_to:expr,
     $color:expr) => {{
        debug_assert!(
            *$kings & $king_from_bb != $crate::bitboard::EMPTY,
            "King not found on expected castling square {}, white:\n{:?}\nblack:\n{:?}",
            $king_from,
            $game.white_kings,
            $game.black_kings
        );
        debug_assert!(
            *$rooks & $rook_from_bb != $crate::bitboard::EMPTY,
            "Rook not found on expected castling square {}, white:\n{:?}\nblack:\n{:?}",
            $rook_from,
            $game.white_rooks,
            $game.black_rooks
        );
        remove_piece!($game, $kings, $king_from_bb, $king_from);
        remove_piece!($game, $rooks, $rook_from_bb, $rook_from);
        add_piece!(
            $game,
            $kings,
            $king_to_bb,
            $king_to,
            PieceType::King,
            $color
        );
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
