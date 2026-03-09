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
    /// Helper method to move a piece from one square to another
    fn move_piece(
        &self,
        game: &mut Game,
        piece: &PieceType,
        color: &PieceColor,
        frombb: BitBoard,
        tobb: BitBoard,
    ) {
        let pieces = game.get_pieces_mut(piece, color);
        *pieces ^= frombb;
        *pieces |= tobb;
    }
}

impl Game {
    /// Plays a move on the board
    pub fn play(&mut self, m: &Move) {
        #[cfg(debug_assertions)]
        {
            let frombb = BitBoard::from_square(m.from(&self.position));
            let (piece, color) = self
                .determine_piece(&frombb)
                .expect("Tried to move nonexistant piece");
            assert_eq!(
                color, self.position.turn,
                "{:?} tried to move {:?}'s {:?} with {}",
                self.position.turn, color, piece, m
            );
        }

        self.capture_position();

        match m {
            Move::Normal { from, to, capture } => {
                let frombb = BitBoard::from_square(*from);
                let tobb = BitBoard::from_square(*to);
                let (piece, color) = self
                    .determine_piece(&frombb)
                    .expect("Couldn't find piece to move!");

                m.move_piece(self, &piece, &color, frombb, tobb);
                if let Some(enemy) = capture {
                    let pieces = self.get_pieces_mut(enemy, &color.opponent());
                    *pieces ^= tobb;
                }

                // Revoking appropriate castling rights
                match piece {
                    PieceType::King => match color {
                        PieceColor::White => {
                            self.position.castling_rights.white_kingside = false;
                            self.position.castling_rights.white_queenside = false;
                        }
                        PieceColor::Black => {
                            self.position.castling_rights.black_kingside = false;
                            self.position.castling_rights.black_queenside = false;
                        }
                    },
                    PieceType::Rook => match *from {
                        Square::A1 => self.position.castling_rights.white_queenside = false,
                        Square::H1 => self.position.castling_rights.white_kingside = false,
                        Square::A8 => self.position.castling_rights.black_queenside = false,
                        Square::H8 => self.position.castling_rights.black_kingside = false,
                        _ => {}
                    },
                    _ => {}
                }

                if *capture == Some(PieceType::Rook) {
                    match *to {
                        Square::A1 => self.position.castling_rights.white_queenside = false,
                        Square::H1 => self.position.castling_rights.white_kingside = false,
                        Square::A8 => self.position.castling_rights.black_queenside = false,
                        Square::H8 => self.position.castling_rights.black_kingside = false,
                        _ => {}
                    }
                }
            }
            Move::CreateEnPassant { at } => {
                let color = self.position.turn;
                let (from, to) = match self.position.turn {
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
                m.move_piece(self, &PieceType::Pawn, &color, frombb, tobb);
            }
            Move::CaptureEnPassant { from: from_file } => {
                let color = self.position.turn;
                let (from, to) = match color {
                    PieceColor::White => (
                        Square::make_square(Rank::Fifth, *from_file),
                        self.position
                            .en_passant_target
                            .expect("CaptureEnPassant played with no en passant target"),
                    ),
                    PieceColor::Black => (
                        Square::make_square(Rank::Fourth, *from_file),
                        self.position
                            .en_passant_target
                            .expect("CaptureEnPassant played with no en passant target"),
                    ),
                };

                let frombb = BitBoard::from_square(from);
                let tobb = BitBoard::from_square(to);
                m.move_piece(self, &PieceType::Pawn, &color, frombb, tobb);

                // Capture the pawn en passant
                let en_passant_bb = BitBoard::from_square(
                    to.backward(&color)
                        .expect("Can't find pawn behind en_passant_target!"),
                );
                let enemy_pawns = self.get_pieces_mut(&PieceType::Pawn, &color.opponent());
                *enemy_pawns ^= en_passant_bb;
            }
            Move::Promotion {
                from: from_file,
                to: to_file,
                piece,
                capture,
            } => {
                let color = self.position.turn;
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

                // Remove pawn from original square
                let pawns = self.get_pieces_mut(&PieceType::Pawn, &color);
                *pawns ^= frombb;

                // Add promoted piece to new square
                let promoted_pieces = self.get_pieces_mut(piece, &color);
                *promoted_pieces |= tobb;

                if let Some(enemy) = capture {
                    let pieces = self.get_pieces_mut(enemy, &color.opponent());
                    *pieces ^= tobb;
                }
            }
            Move::Castle { side } => match &self.position.turn {
                PieceColor::White => {
                    self.position.castling_rights.white_queenside = false;
                    self.position.castling_rights.white_kingside = false;

                    match side {
                        CastleSide::Queenside => {
                            self.position.white_kings ^=
                                castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                            self.position.white_rooks ^=
                                castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            self.position.white_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                            self.position.white_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    }
                }
                PieceColor::Black => {
                    self.position.castling_rights.black_queenside = false;
                    self.position.castling_rights.black_kingside = false;

                    match side {
                        CastleSide::Queenside => {
                            self.position.black_kings ^=
                                castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                            self.position.black_rooks ^=
                                castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            self.position.black_kings ^= castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                            self.position.black_rooks ^= castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
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
    use crate::board::Board;
    use crate::castling::CastleSide;
    use crate::file::File;
    use crate::game::Game;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

    #[test]
    fn both_lose_castling_rights_by_moving_kings() {
        let mut game = Game::from_position(
            Board::from_fen("rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1")
                .unwrap(),
        );

        let black_moves = Move::new(Square::E8, Square::D7, &game.position);
        game.play(&black_moves);

        compare_to_fen(
            &game.position,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQ - 1 2",
        );

        let white_moves = Move::new(Square::E1, Square::E2, &game.position);
        game.play(&white_moves);

        compare_to_fen(
            &game.position,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP1KPPP/RNBQ1B1R b - - 2 2",
        );
    }

    #[test]
    fn both_lose_castling_rights_by_moving_rooks() {
        let mut game = Game::from_position(
            Board::from_fen("rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1")
                .unwrap(),
        );

        let black_moves = Move::new(Square::H8, Square::G8, &game.position);
        game.play(&black_moves);

        compare_to_fen(
            &game.position,
            "rnbqkbr1/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQq - 1 2",
        );

        let white_moves = Move::new(Square::H1, Square::G1, &game.position);
        game.play(&white_moves);

        compare_to_fen(
            &game.position,
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
        let mut game = Game::from_position(Board::from_fen(fen_before).unwrap());

        let moves = Square::E1.king_psuedo_legal_moves(&game);
        should_generate(&moves, &to_play);

        game.play(&to_play);
        compare_to_fen(&game.position, fen_after);
    }

    #[test]
    fn black_king_castles_kingside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let fen_after = "rn3rk1/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR w - - 8 8";
        let to_play = Move::Castle {
            side: CastleSide::Kingside,
        };
        let mut game = Game::from_position(Board::from_fen(fen_before).unwrap());

        let moves = Square::E8.king_psuedo_legal_moves(&game);
        should_generate(&moves, &to_play);

        game.play(&to_play);
        compare_to_fen(&game.position, fen_after);
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
        let looking_for_from = looking_for.from(&game.position);

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
            let m = Move::new(from, to, &game.position);
            game.play(&m);
        }

        assert_eq!(game.position.turn, PieceColor::White);
        let moves = looking_for
            .from(&game.position)
            .pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );

        println!("{}", game.position.to_fen());
        game.play(&looking_for);
        println!("{}", game.position.to_fen());

        assert_eq!(game.position.turn, PieceColor::Black);
        assert!(
            Square::H8.in_bitboard(&game.position.white_queens),
            "Expected white queen at H8 after promotion"
        );
        assert!(
            !Square::H8.in_bitboard(&game.position.white_pawns),
            "H8 incorrectly contains a white pawn after promotion"
        );
        assert!(
            !looking_for_from.in_bitboard(&game.position.white_pawns),
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
        let original = game.position.clone();
        game.play(&pawn);
        let after_pawn = game.position.clone();

        game = Game::default();
        game.play(&Move::new(Square::E2, Square::E3, &game.position));
        game.play(&knight);
        let after_knight = game.position.clone();

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        game = Game::from_position(Board::from_fen(fen).unwrap());
        game.play(&king);
        let after_king = game.position.clone();

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
        let white_pawns_before = game.position.white_pawns.popcnt();
        let black_pawns_before = game.position.black_pawns.popcnt();

        for (from, to) in [(Square::B2, Square::B4), (Square::C7, Square::C5)] {
            let m = Move::new(from, to, &game.position);
            game.play(&m);
        }

        let capture = Move::new(Square::B4, Square::C5, &game.position);
        game.play(&capture);

        let white_pawns_after = game.position.white_pawns.popcnt();
        let black_pawns_after = game.position.black_pawns.popcnt();

        assert!(
            !Square::B2.in_bitboard(&game.position.white_pawns),
            "White never moved"
        );
        assert!(
            !capture
                .from(&game.position)
                .in_bitboard(&game.position.white_pawns),
            "White moved but failed to capture"
        );
        assert!(
            !capture
                .to(&game.position)
                .in_bitboard(&game.position.black_pawns),
            "The black pawn is still standing"
        );
        assert!(
            capture
                .to(&game.position)
                .in_bitboard(&game.position.white_pawns),
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

        assert_eq!(game.position.turn, PieceColor::Black);
        assert!(
            capture
                .from(&game.position)
                .in_bitboard(&game.position.black_pawns),
            "Black pawn not in position"
        );

        let moves = capture.from(&game.position).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&capture),
            "Black pawn doesn't see en passant target. {}",
            format_pretty_list(&moves)
        );

        let white_pawns_before = game.position.white_pawns.popcnt();
        let black_pawns_before = game.position.black_pawns.popcnt();
        game.play(&capture);
        let white_pawns_after = game.position.white_pawns.popcnt();
        let black_pawns_after = game.position.black_pawns.popcnt();

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
        assert_eq!(game.position.en_passant_target, None);
        game.play(&m);
        assert_eq!(game.position.en_passant_target, Some(Square::E3));
    }
}
