use crate::{
    add_piece,
    bitboard::BitBoard,
    castle,
    castling::{self, CastleSide},
    game::Game,
    get_pieces_mut,
    movegen::{
        moves::Move,
        pieces::piece::{PieceColor, PieceType},
    },
    rank::Rank,
    remove_piece,
    square::Square,
};

impl Game {
    /// Plays a move on the board
    pub fn play(&mut self, m: &Move) {
        #[cfg(debug_assertions)]
        {
            let from = m.from(self);
            let (piece, color) = self
                .piece_lookup(from)
                .expect("Tried to move nonexistant piece");
            assert_eq!(
                color, self.turn,
                "{:?} tried to move {:?}'s {:?} with {}",
                self.turn, color, piece, m
            );
        }

        self.capture_position();

        match m {
            Move::Normal { from, to, capture } => {
                let frombb = BitBoard::from_square(*from);
                let tobb = BitBoard::from_square(*to);
                let (piece, color) = self
                    .piece_lookup(*from)
                    .expect("Couldn't find piece to move!");

                if let Some(enemy) = capture {
                    let pieces = get_pieces_mut!(self, enemy, &color.opponent());
                    remove_piece!(self, pieces, tobb, *to);
                }

                let pieces = get_pieces_mut!(self, &piece, &color);
                remove_piece!(self, pieces, frombb, *from);
                add_piece!(self, pieces, tobb, *to, piece, color);

                // Revoking appropriate castling rights
                macro_rules! revoke_castling_rights {
                    ($sq:expr) => {
                        match $sq {
                            castling::BLACK_CASTLE_KINGSIDE_ROOK_FROM => {
                                self.castling_rights.revoke_black_kingside()
                            }
                            castling::BLACK_CASTLE_QUEENSIDE_ROOK_FROM => {
                                self.castling_rights.revoke_black_queenside()
                            }
                            castling::WHITE_CASTLE_KINGSIDE_ROOK_FROM => {
                                self.castling_rights.revoke_white_kingside()
                            }
                            castling::WHITE_CASTLE_QUEENSIDE_ROOK_FROM => {
                                self.castling_rights.revoke_white_queenside()
                            }
                            _ => {}
                        }
                    };
                }

                match piece {
                    PieceType::King => match color {
                        PieceColor::White => {
                            self.castling_rights.revoke_white();
                        }
                        PieceColor::Black => {
                            self.castling_rights.revoke_black();
                        }
                    },
                    PieceType::Rook => revoke_castling_rights!(*from),
                    _ => {}
                }

                if *capture == Some(PieceType::Rook) {
                    revoke_castling_rights!(*to);
                }
            }
            Move::CreateEnPassant { at } => {
                let color = self.turn;
                let (from, to) = match self.turn {
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
                let pieces = get_pieces_mut!(self, &PieceType::Pawn, &color);
                remove_piece!(self, pieces, frombb, from);
                add_piece!(self, pieces, tobb, to, PieceType::Pawn, color);
            }
            Move::CaptureEnPassant { from: from_file } => {
                let color = self.turn;
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Fifth, *from_file),
                        self.en_passant_target
                            .expect("CaptureEnPassant played with no en passant target"),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Fourth, *from_file),
                        self.en_passant_target
                            .expect("CaptureEnPassant played with no en passant target"),
                    ),
                };

                // Capture the pawn en passant
                let en_passant_sq = to
                    .backward(&color)
                    .expect("Can't find pawn behind en_passant_target!");
                let en_passant_bb = BitBoard::from_square(en_passant_sq);

                let pieces = get_pieces_mut!(self, &PieceType::Pawn, &color.opponent());
                remove_piece!(self, pieces, en_passant_bb, en_passant_sq);

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);
                let pieces = get_pieces_mut!(self, &PieceType::Pawn, &color);
                remove_piece!(self, pieces, frombb, from);
                add_piece!(self, pieces, tobb, to, PieceType::Pawn, color);
            }
            Move::Promotion {
                from: from_file,
                to: to_file,
                piece,
                capture,
            } => {
                let color = self.turn;
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Seventh, *from_file),
                        Square::make_square(Rank::Eighth, *to_file),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Second, *from_file),
                        Square::make_square(Rank::First, *to_file),
                    ),
                };

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);

                if let Some(enemy) = capture {
                    let pieces = get_pieces_mut!(self, enemy, &color.opponent());
                    remove_piece!(self, pieces, tobb, to);
                }

                // Remove pawn from original square
                let pawns = get_pieces_mut!(self, &PieceType::Pawn, &color);
                remove_piece!(self, pawns, frombb, from);

                // Add promoted piece to new square
                let promoted_pieces = get_pieces_mut!(self, piece, &color);
                add_piece!(self, promoted_pieces, tobb, to, *piece, color);
            }
            Move::Castle { side } => match &self.turn {
                PieceColor::White => {
                    self.castling_rights.revoke_white();

                    match side {
                        CastleSide::Queenside => castle!(
                            self,
                            &mut self.white_kings,
                            &mut self.white_rooks,
                            castling::WHITE_CASTLE_QUEENSIDE_KING_FROM_BB,
                            castling::WHITE_CASTLE_QUEENSIDE_KING_FROM,
                            castling::WHITE_CASTLE_QUEENSIDE_KING_TO_BB,
                            castling::WHITE_CASTLE_QUEENSIDE_KING_TO,
                            castling::WHITE_CASTLE_QUEENSIDE_ROOK_FROM_BB,
                            castling::WHITE_CASTLE_QUEENSIDE_ROOK_FROM,
                            castling::WHITE_CASTLE_QUEENSIDE_ROOK_TO_BB,
                            castling::WHITE_CASTLE_QUEENSIDE_ROOK_TO,
                            PieceColor::White
                        ),
                        CastleSide::Kingside => castle!(
                            self,
                            &mut self.white_kings,
                            &mut self.white_rooks,
                            castling::WHITE_CASTLE_KINGSIDE_KING_FROM_BB,
                            castling::WHITE_CASTLE_KINGSIDE_KING_FROM,
                            castling::WHITE_CASTLE_KINGSIDE_KING_TO_BB,
                            castling::WHITE_CASTLE_KINGSIDE_KING_TO,
                            castling::WHITE_CASTLE_KINGSIDE_ROOK_FROM_BB,
                            castling::WHITE_CASTLE_KINGSIDE_ROOK_FROM,
                            castling::WHITE_CASTLE_KINGSIDE_ROOK_TO_BB,
                            castling::WHITE_CASTLE_KINGSIDE_ROOK_TO,
                            PieceColor::White
                        ),
                    }
                }
                PieceColor::Black => {
                    self.castling_rights.revoke_black();

                    match side {
                        CastleSide::Queenside => castle!(
                            self,
                            &mut self.black_kings,
                            &mut self.black_rooks,
                            castling::BLACK_CASTLE_QUEENSIDE_KING_FROM_BB,
                            castling::BLACK_CASTLE_QUEENSIDE_KING_FROM,
                            castling::BLACK_CASTLE_QUEENSIDE_KING_TO_BB,
                            castling::BLACK_CASTLE_QUEENSIDE_KING_TO,
                            castling::BLACK_CASTLE_QUEENSIDE_ROOK_FROM_BB,
                            castling::BLACK_CASTLE_QUEENSIDE_ROOK_FROM,
                            castling::BLACK_CASTLE_QUEENSIDE_ROOK_TO_BB,
                            castling::BLACK_CASTLE_QUEENSIDE_ROOK_TO,
                            PieceColor::Black
                        ),
                        CastleSide::Kingside => castle!(
                            self,
                            &mut self.black_kings,
                            &mut self.black_rooks,
                            castling::BLACK_CASTLE_KINGSIDE_KING_FROM_BB,
                            castling::BLACK_CASTLE_KINGSIDE_KING_FROM,
                            castling::BLACK_CASTLE_KINGSIDE_KING_TO_BB,
                            castling::BLACK_CASTLE_KINGSIDE_KING_TO,
                            castling::BLACK_CASTLE_KINGSIDE_ROOK_FROM_BB,
                            castling::BLACK_CASTLE_KINGSIDE_ROOK_FROM,
                            castling::BLACK_CASTLE_KINGSIDE_ROOK_TO_BB,
                            castling::BLACK_CASTLE_KINGSIDE_ROOK_TO,
                            PieceColor::Black
                        ),
                    }
                }
            },
        }

        self.next_turn(m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::castling::CastleSide;
    use crate::file::File;
    use crate::game::Game;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

    #[test]
    fn both_lose_castling_rights_by_moving_kings() {
        let fen = "rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let black_moves = Move::infer(Square::E8, Square::D7, &game);
        game.play(&black_moves);

        compare_to_fen(
            &game,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQ - 1 2",
        );

        let white_moves = Move::infer(Square::E1, Square::E2, &game);
        game.play(&white_moves);

        compare_to_fen(
            &game,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP1KPPP/RNBQ1B1R b - - 2 2",
        );
    }

    #[test]
    fn both_lose_castling_rights_by_moving_rooks() {
        let fen = "rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let black_moves = Move::infer(Square::H8, Square::G8, &game);
        game.play(&black_moves);

        compare_to_fen(
            &game,
            "rnbqkbr1/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQq - 1 2",
        );

        let white_moves = Move::infer(Square::H1, Square::G1, &game);
        game.play(&white_moves);

        compare_to_fen(
            &game,
            "rnbqkbr1/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKBR1 b Qq - 2 2",
        );
    }

    #[test]
    fn white_king_castles_queenside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/R3K1NR w KQkq - 6 7";
        let fen_after = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let to_play = Move::Castle {
            side: CastleSide::Queenside,
        };
        let mut game = Game::from_fen(fen_before).unwrap();

        let moves = Square::E1.king_psuedo_legal_moves(&game);
        should_generate(&moves, &to_play);

        game.play(&to_play);
        compare_to_fen(&game, fen_after);
    }

    #[test]
    fn black_king_castles_kingside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let fen_after = "rn3rk1/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR w - - 8 8";
        let to_play = Move::Castle {
            side: CastleSide::Kingside,
        };
        let mut game = Game::from_fen(fen_before).unwrap();

        let moves = Square::E8.king_psuedo_legal_moves(&game);
        should_generate(&moves, &to_play);

        game.play(&to_play);
        compare_to_fen(&game, fen_after);
    }

    #[test]
    fn white_pawn_promotes_to_queen() {
        let mut game = Game::default();
        let looking_for = Move::Promotion {
            from: File::G,
            to: File::H,
            piece: PieceType::Queen,
            capture: Some(PieceType::Rook),
        };
        let looking_for_from = looking_for.from(&game);

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
        let moves = looking_for.from(&game).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );

        println!("{}", game.to_fen());
        game.play(&looking_for);
        println!("{}", game.to_fen());

        assert_eq!(game.turn, PieceColor::Black);
        assert!(
            Square::H8.in_bitboard(&game.white_queens),
            "Expected white queen at H8 after promotion"
        );
        assert!(
            !Square::H8.in_bitboard(&game.white_pawns),
            "H8 incorrectly contains a white pawn after promotion"
        );
        assert!(
            !looking_for_from.in_bitboard(&game.white_pawns),
            "Original white pawn still present at {} after promotion",
            looking_for_from
        );
    }

    #[test]
    fn make_moves() {
        let pawn = Move::Normal {
            from: Square::C2,
            to: Square::C3,
            capture: None,
        };
        let knight = Move::Normal {
            from: Square::G8,
            to: Square::F6,
            capture: None,
        };
        let king = Move::Normal {
            from: Square::E1,
            to: Square::E2,
            capture: None,
        };

        let mut game = Game::default();
        let original = game.clone();
        game.play(&pawn);
        let after_pawn = game.clone();

        game = Game::default();
        game.play(&Move::infer(Square::E2, Square::E3, &game));
        game.play(&knight);
        let after_knight = game.clone();

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        game = Game::from_fen(fen).unwrap();
        game.play(&king);
        let after_king = game.clone();

        assert!(pawn.from(&original).in_bitboard(&original.white_pawns));
        assert!(!pawn.to(&original).in_bitboard(&original.white_pawns));
        assert!(!pawn.from(&after_pawn).in_bitboard(&after_pawn.white_pawns));
        assert!(pawn.to(&after_pawn).in_bitboard(&after_pawn.white_pawns));

        assert!(knight.from(&original).in_bitboard(&original.black_knights));
        assert!(!knight.to(&original).in_bitboard(&original.black_knights));
        assert!(
            !knight
                .from(&after_knight)
                .in_bitboard(&after_knight.black_knights)
        );
        assert!(
            knight
                .to(&after_knight)
                .in_bitboard(&after_knight.black_knights)
        );

        assert!(king.from(&original).in_bitboard(&original.white_kings));
        assert!(!king.to(&original).in_bitboard(&original.white_kings));
        assert!(!king.from(&after_king).in_bitboard(&after_king.white_kings));
        assert!(king.to(&after_king).in_bitboard(&after_king.white_kings));
    }

    #[test]
    fn white_pawn_captures_black_pawn() {
        let mut game = Game::default();
        let white_pawns_before = game.white_pawns.popcnt();
        let black_pawns_before = game.black_pawns.popcnt();

        for (from, to) in [(Square::B2, Square::B4), (Square::C7, Square::C5)] {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        let capture = Move::infer(Square::B4, Square::C5, &game);
        game.play(&capture);

        let white_pawns_after = game.white_pawns.popcnt();
        let black_pawns_after = game.black_pawns.popcnt();

        assert!(
            !Square::B2.in_bitboard(&game.white_pawns),
            "White never moved"
        );
        assert!(
            !capture.from(&game).in_bitboard(&game.white_pawns),
            "White moved but failed to capture"
        );
        assert!(
            !capture.to(&game).in_bitboard(&game.black_pawns),
            "The black pawn is still standing"
        );
        assert!(
            capture.to(&game).in_bitboard(&game.white_pawns),
            "White isn't in the correct position"
        );

        assert_ne!(
            black_pawns_before, black_pawns_after,
            "The number of black pawns didn't change"
        );
        assert_eq!(
            black_pawns_before - 1,
            black_pawns_after,
            "The number of black pawns didn't decrease by one"
        );
        assert_eq!(
            white_pawns_before, white_pawns_after,
            "The number of white pawns changed"
        );
    }

    #[test]
    fn black_pawn_takes_en_passant_target() {
        let mut game = Game::default();
        let capture = Move::CaptureEnPassant {
            from: Square::B4.get_file(),
        };
        for m in [
            Move::Normal {
                from: Square::D2,
                to: Square::D3,
                capture: None,
            },
            Move::CreateEnPassant {
                at: Square::B7.get_file(),
            },
            Move::Normal {
                from: Square::D3,
                to: Square::D4,
                capture: None,
            },
            Move::Normal {
                from: Square::B5,
                to: Square::B4,
                capture: None,
            },
            Move::CreateEnPassant {
                at: Square::C2.get_file(),
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::Black);
        assert!(
            capture.from(&game).in_bitboard(&game.black_pawns),
            "Black pawn not in position"
        );

        let moves = capture.from(&game).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&capture),
            "Black pawn doesn't see en passant target. {}",
            format_pretty_list(&moves)
        );

        let white_pawns_before = game.white_pawns.popcnt();
        let black_pawns_before = game.black_pawns.popcnt();
        game.play(&capture);
        let white_pawns_after = game.white_pawns.popcnt();
        let black_pawns_after = game.black_pawns.popcnt();

        assert_eq!(black_pawns_before, black_pawns_after);
        assert_eq!(
            white_pawns_before - 1,
            white_pawns_after,
            "The white target is still standing"
        );
    }

    #[test]
    fn en_passant_target_is_created() {
        let mut game = Game::default();
        let m = Move::CreateEnPassant { at: File::E };
        assert_eq!(game.en_passant_target, None);
        game.play(&m);
        assert_eq!(game.en_passant_target, Some(Square::E3));
    }

    #[test]
    fn piece_table_is_updated() {
        let mut game = Game::default();
        let from = Square::E2;
        let to = Square::E4;
        let m = Move::infer(from, to, &game);
        let pawn = Some((PieceType::Pawn, PieceColor::White));

        assert_eq!(game.piece_lookup(from), pawn);

        game.play(&m);

        assert_ne!(
            game.piece_lookup(from),
            pawn,
            "There is still a pawn on {from} after playing {m}"
        );

        assert_eq!(
            game.piece_lookup(from),
            None,
            "Something is still occupying {from} after playing {m}"
        );

        assert_eq!(
            game.piece_lookup(to),
            pawn,
            "The pawn is not on {to} after playing {m}"
        );

        game.unplay(&m);

        assert_eq!(
            game.piece_lookup(from),
            pawn,
            "The pawn was not moved back to {from}"
        );
        assert_eq!(game.piece_lookup(to), None, "Something is still in {to}");
    }
}
