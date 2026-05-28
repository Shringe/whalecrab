use crate::{
    bitboard::BitBoard,
    file::File,
    movegen::{
        moves::{Move, targets_to_moves},
        pieces::piece::{PieceColor, PieceMoveInfo, PieceType},
    },
    position::game::Game,
    rank::Rank,
    square::Square,
    vectors::Vector,
};

macro_rules! assert_shift {
    ($shifted:expr, $regular:expr) => {
        #[cfg(debug_assertions)]
        {
            let sqbb = match $regular {
                Some(sq) => BitBoard::from_square(sq),
                None => crate::bitboard::EMPTY,
            };

            assert_eq!($shifted, sqbb);
        }
    };
}

pub const MAXIMUM_MOVE_COUNT: u32 = 4;

pub fn push_psuedo_legal_moves_white<V: Vector<Move>>(moves: &mut V, game: &Game) {
    let twice_mask = Rank::Fourth.mask();
    let promotion_mask = Rank::Eighth.mask();
    let unoccupied = !game.occupied;

    let once = game.white_pawns.up() & unoccupied;
    let twice = once.up() & unoccupied & twice_mask;
    let promotions = once & promotion_mask;

    let capture_right = game.white_pawns.up_right() & (game.black_occupied & !File::A.mask());
    let capture_left = game.white_pawns.up_left() & (game.black_occupied & !File::H.mask());

    macro_rules! get_piece {
        ($sq:expr) => {
            Some(
                if cfg!(debug_assertions) {
                    game.piece_lookup($sq).unwrap()
                } else {
                    // Should be safe because with pawn move generation we know for sure
                    // whether or not we can capture ahead of time using bit manipulation
                    unsafe { game.piece_lookup($sq).unwrap_unchecked() }
                }
                .0,
            )
        };
    }

    for to in once ^ promotions {
        let from = unsafe { to.down_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: None,
        };
        moves.push(m);
    }

    for sq in twice {
        let m = Move::CreateEnPassant { at: sq.get_file() };
        moves.push(m);
    }

    for sq in promotions {
        let file = sq.get_file();
        let m = Move::Promotion {
            from: file,
            to: file,
            piece: PieceType::Queen,
            capture: None,
        };
        moves.push(m);
    }

    for to in capture_right & !promotion_mask {
        let from = unsafe { to.dleft_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_left & !promotion_mask {
        let from = unsafe { to.dright_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_right & promotion_mask {
        let from = unsafe { to.dleft_unchecked() };
        let m = Move::Promotion {
            from: from.get_file(),
            to: to.get_file(),
            piece: PieceType::Queen,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_left & promotion_mask {
        let from = unsafe { to.dright_unchecked() };
        let m = Move::Promotion {
            from: from.get_file(),
            to: to.get_file(),
            piece: PieceType::Queen,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    if let Some(target) = game.en_passant_target {
        let mut process_target = |sq: Option<Square>| {
            if let Some(sq) = sq
                && game.white_pawns.has_square(BitBoard::from_square(sq))
            {
                let m = Move::CaptureEnPassant {
                    from: sq.get_file(),
                };
                moves.push(m);
            }
        };
        process_target(target.dleft());
        process_target(target.dright());
    }
}

pub fn push_psuedo_legal_moves_black<V: Vector<Move>>(moves: &mut V, game: &Game) {
    let twice_mask = Rank::Fifth.mask();
    let promotion_mask = Rank::First.mask();
    let unoccupied = !game.occupied;

    let once = game.black_pawns.down() & unoccupied;
    let twice = once.down() & unoccupied & twice_mask;
    let promotions = once & promotion_mask;

    let capture_right = game.black_pawns.down_left() & (game.white_occupied & !File::H.mask());
    let capture_left = game.black_pawns.down_right() & (game.white_occupied & !File::A.mask());

    macro_rules! get_piece {
        ($sq:expr) => {
            Some(
                if cfg!(debug_assertions) {
                    game.piece_lookup($sq).unwrap()
                } else {
                    unsafe { game.piece_lookup($sq).unwrap_unchecked() }
                }
                .0,
            )
        };
    }

    for to in once ^ promotions {
        let from = unsafe { to.up_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: None,
        };
        moves.push(m);
    }

    for sq in twice {
        let m = Move::CreateEnPassant { at: sq.get_file() };
        moves.push(m);
    }

    for sq in promotions {
        let file = sq.get_file();
        let m = Move::Promotion {
            from: file,
            to: file,
            piece: PieceType::Queen,
            capture: None,
        };
        moves.push(m);
    }

    for to in capture_right & !promotion_mask {
        let from = unsafe { to.uright_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_left & !promotion_mask {
        let from = unsafe { to.uleft_unchecked() };
        let m = Move::Normal {
            from,
            to,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_right & promotion_mask {
        let from = unsafe { to.uright_unchecked() };
        let m = Move::Promotion {
            from: from.get_file(),
            to: to.get_file(),
            piece: PieceType::Queen,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    for to in capture_left & promotion_mask {
        let from = unsafe { to.uleft_unchecked() };
        let m = Move::Promotion {
            from: from.get_file(),
            to: to.get_file(),
            piece: PieceType::Queen,
            capture: get_piece!(to),
        };
        moves.push(m);
    }

    if let Some(target) = game.en_passant_target {
        let mut process_target = |sq: Option<Square>| {
            if let Some(sq) = sq
                && game.black_pawns.has_square(BitBoard::from_square(sq))
            {
                let m = Move::CaptureEnPassant {
                    from: sq.get_file(),
                };
                moves.push(m);
            }
        };
        process_target(target.uleft());
        process_target(target.uright());
    }
}

impl Square {
    /// Generates all psuedo legal moves for a single pawn
    /// En_Passant is considered
    /// Promotion is considered (only for queen)
    /// King safety not considered
    pub fn pawn_psuedo_legal_moves(self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.pawn_psuedo_legal_targets(game).targets, self, game)
    }

    pub fn pawn_psuedo_legal_targets(self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let sqbb = BitBoard::from_square(self);
        let friendly = game
            .determine_color(BitBoard::from_square(self))
            .expect("Tried to move non existent pawn");

        let file = self.get_file();

        match friendly {
            PieceColor::White => {
                let oncebb = sqbb << 8;
                assert_shift!(oncebb, self.up());

                if !game.occupied.has_square(oncebb) {
                    moveinfo.targets |= oncebb;
                    if self.get_rank() == Rank::Second {
                        let twicebb = oncebb << 8;
                        assert_shift!(twicebb, self.up().and_then(|s| s.up()));

                        if !game.occupied.has_square(twicebb) {
                            moveinfo.targets |= twicebb;
                        }
                    }
                }

                if file > File::A {
                    let uleft = sqbb << 7;
                    assert_shift!(uleft, self.uleft());

                    moveinfo.attacks |= uleft;
                    if game.black_occupied.has_square(uleft) {
                        moveinfo.targets |= uleft;
                    } else if let Some(target) = game.en_passant_target
                        && uleft == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= uleft;
                    }
                }

                if file < File::H {
                    let uright = sqbb << 9;
                    assert_shift!(uright, self.uright());

                    moveinfo.attacks |= uright;
                    if game.black_occupied.has_square(uright) {
                        moveinfo.targets |= uright;
                    } else if let Some(target) = game.en_passant_target
                        && uright == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= uright;
                    }
                }
            }

            PieceColor::Black => {
                let oncebb = sqbb >> 8;
                assert_shift!(oncebb, self.down());

                if !game.occupied.has_square(oncebb) {
                    moveinfo.targets |= oncebb;
                    if self.get_rank() == Rank::Seventh {
                        let twicebb = oncebb >> 8;
                        assert_shift!(twicebb, self.down().and_then(|s| s.down()));

                        if !game.occupied.has_square(twicebb) {
                            moveinfo.targets |= twicebb;
                        }
                    }
                }

                if file > File::A {
                    let dleft = sqbb >> 9;
                    assert_shift!(dleft, self.dleft());

                    moveinfo.attacks |= dleft;
                    if game.white_occupied.has_square(dleft) {
                        moveinfo.targets |= dleft;
                    } else if let Some(target) = game.en_passant_target
                        && dleft == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= dleft;
                    }
                }

                if file < File::H {
                    let dright = sqbb >> 7;
                    assert_shift!(dright, self.dright());

                    moveinfo.attacks |= dright;
                    if game.white_occupied.has_square(dright) {
                        moveinfo.targets |= dright;
                    } else if let Some(target) = game.en_passant_target
                        && dright == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= dright;
                    }
                }
            }
        }

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::File, movegen::pieces::piece::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_pawn_sees_black_target() {
        let mut game = Game::default();
        let looking_for = Move::Normal {
            from: Square::H4,
            to: Square::G5,
            capture: Some(PieceType::Pawn),
        };

        for m in [
            Move::Normal {
                from: Square::H2,
                to: Square::H4,
                capture: None,
            },
            Move::Normal {
                from: Square::G7,
                to: Square::G5,
                capture: None,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::White);
        let moves = looking_for.from(game.turn).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn black_pawn_sees_white_target() {
        let mut game = Game::default();
        let looking_for = Move::Normal {
            from: Square::D5,
            to: Square::C4,
            capture: Some(PieceType::Pawn),
        };

        for m in [
            Move::CreateEnPassant {
                at: Square::C2.get_file(),
            },
            Move::Normal {
                from: Square::D7,
                to: Square::D5,
                capture: None,
            },
            Move::Normal {
                from: Square::H2,
                to: Square::H3,
                capture: None,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::Black);
        let moves = looking_for.from(game.turn).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "Black pawn can't see target. Available moves: {:?}",
            moves
        );
    }

    #[test]
    fn white_pawn_sees_queen_promotion() {
        let mut game = Game::default();
        let looking_for = Move::Promotion {
            from: File::G,
            to: File::H,
            piece: PieceType::Queen,
            capture: Some(PieceType::Rook),
        };

        for (from, to) in [
            (Square::H2, Square::H4),
            (Square::G7, Square::G5),
            (Square::H4, Square::G5),
            (Square::H7, Square::H6),
            (Square::G5, Square::H6),
            (Square::F8, Square::G7),
            (Square::H6, Square::G7),
            (Square::E7, Square::E5),
        ] {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::White);
        let moves = looking_for.from(game.turn).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
