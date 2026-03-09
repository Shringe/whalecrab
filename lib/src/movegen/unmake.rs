use crate::{
    bitboard::BitBoard,
    castling::{self, CastleSide},
    game::Game,
    movegen::{
        moves::Move,
        pieces::piece::{PieceColor, PieceType},
    },
    rank::Rank,
    square::Square,
};

impl Move {
    /// Unplays a move on the board.
    pub fn unplay(&self, game: &mut Game) {
        match self {
            Move::Normal { from, to, capture } => {
                let frombb = BitBoard::from_square(*from);
                let tobb = BitBoard::from_square(*to);
                let (piece, color) = game
                    .determine_piece(&tobb)
                    .expect("Couldn't find piece to unmove!");

                let pieces = game.get_pieces_mut(&piece, &color);
                *pieces ^= tobb;
                *pieces |= frombb;

                if let Some(enemy) = capture {
                    let pieces = game.get_pieces_mut(enemy, &color.opponent());
                    *pieces |= tobb;
                }
            }
            Move::CreateEnPassant { at } => {
                let color = game.position.turn.opponent();
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Second, *at),
                        Square::make_square(Rank::Fourth, *at),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Seventh, *at),
                        Square::make_square(Rank::Fifth, *at),
                    ),
                };

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);
                let pawns = game.get_pieces_mut(&PieceType::Pawn, &color);
                *pawns ^= tobb;
                *pawns |= frombb;
            }
            Move::CaptureEnPassant { from: from_file } => {
                let color = game.position.turn.opponent();
                let enemy_color = color.opponent();
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Fifth, *from_file),
                        game.position
                            .en_passant_target
                            .expect("CaptureEnPassant unplayed with no en passant target"),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Fourth, *from_file),
                        game.position
                            .en_passant_target
                            .expect("CaptureEnPassant unplayed with no en passant target"),
                    ),
                };

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);

                let pawns = game.get_pieces_mut(&PieceType::Pawn, &color);
                *pawns ^= tobb;
                *pawns |= frombb;

                // Restore the captured pawn
                let captured_pawn_bb = BitBoard::from_square(
                    to.backward(&color)
                        .expect("Can't find pawn behind en_passant_target!"),
                );
                let enemy_pawns = game.get_pieces_mut(&PieceType::Pawn, &enemy_color);
                *enemy_pawns |= captured_pawn_bb;
            }
            Move::Promotion { at, piece, capture } => {
                let color = game.position.turn.opponent();
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Seventh, *at),
                        Square::make_square(Rank::Eighth, *at),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Second, *at),
                        Square::make_square(Rank::First, *at),
                    ),
                };

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);

                // Remove promoted piece from destination square
                let promoted_pieces = game.get_pieces_mut(piece, &color);
                *promoted_pieces ^= tobb;

                // Restore original pawn
                let pawns = game.get_pieces_mut(&PieceType::Pawn, &color);
                *pawns |= frombb;

                if let Some(enemy) = capture {
                    let pieces = game.get_pieces_mut(enemy, &color.opponent());
                    *pieces |= tobb;
                }
            }
            Move::Castle { side } => {
                let color = game.position.turn.opponent();
                match color {
                    PieceColor::White => match side {
                        CastleSide::Queenside => {
                            game.position.white_kings ^=
                                castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                            game.position.white_rooks ^=
                                castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            game.position.white_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                            game.position.white_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    },
                    PieceColor::Black => match side {
                        CastleSide::Queenside => {
                            game.position.black_kings ^=
                                castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                            game.position.black_rooks ^=
                                castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            game.position.black_kings ^= castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                            game.position.black_rooks ^= castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    },
                }
            }
        }

        game.restore_position();
        game.previous_turn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, State};
    use crate::square::Square;
    use crate::test_utils::compare_games;

    macro_rules! play_unplay_with_game {
        ($game:expr, $sequence:expr) => {{
            let mut game = $game;
            let before = game.clone();
            let mut has_played = Vec::new();
            for (from, to) in $sequence {
                let m = Move::new(from, to, &game.position);
                m.play(&mut game);
                has_played.push(m);
            }

            has_played.reverse();
            for m in has_played {
                m.unplay(&mut game);
            }

            compare_games(&before, &game);
        }};
    }

    macro_rules! play_unplay {
        ($sequence:expr) => {{
            let game = Game::default();
            play_unplay_with_game!(game, $sequence)
        }};
        ($fen:expr, $sequence:expr) => {{
            let game = Game::from_position(Board::from_fen($fen).unwrap());
            play_unplay_with_game!(game, $sequence)
        }};
    }

    macro_rules! test_play_unplay {
        ($test_name:ident, $sequence:expr) => {
            #[test]
            fn $test_name() {
                play_unplay!($sequence);
            }
        };
        ($test_name:ident, $fen:expr, $sequence:expr) => {
            #[test]
            fn $test_name() {
                play_unplay!($fen, $sequence);
            }
        };
    }

    test_play_unplay!(unplay_normal, [(Square::G1, Square::F3)]);
    test_play_unplay!(unplay_create_en_passant, [(Square::E2, Square::E4)]);

    test_play_unplay!(
        play_unplay_large_sequence,
        [
            (Square::E2, Square::E4),
            (Square::D7, Square::D5),
            // (Square::E4, Square::E5),
            // (Square::D5, Square::D4),
            // (Square::C2, Square::C4),
            // (Square::D4, Square::C3), // En passant capture
        ]
    );

    test_play_unplay!(
        unplay_capture,
        "rnbqkb1r/pppppp2/8/8/8/8/PPPPP3/RNBQK2R b KQkq - 0 1",
        [(Square::H8, Square::H1)]
    );

    test_play_unplay!(
        unplay_capture_en_passant,
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 2",
        [(Square::E5, Square::F6)]
    );

    test_play_unplay!(
        unplay_promotion,
        "8/8/8/8/8/8/5Kpk/8 b - - 0 1",
        [(Square::G2, Square::G1)]
    );

    test_play_unplay!(
        unplay_castle,
        "rnbqkbnr/pppppppp/8/8/2BPPB2/P1N2N1P/1PPQ1PP1/R3K2R w KQkq - 0 1",
        [(Square::E1, Square::C1)]
    );

    test_play_unplay!(
        unplay_promotion_with_capture,
        "5q2/6P1/8/8/8/6rr/RR6/KN4nk w - - 0 1",
        [(Square::G7, Square::F8)]
    );

    #[test]
    fn no_repetition() {
        let mut game = Game::default();
        let m = Move::new(Square::E2, Square::E4, &game.position);
        for _ in 0..5 {
            m.play(&mut game);
            m.unplay(&mut game);
            assert_eq!(game.position.state, State::InProgress);
        }
    }
}
