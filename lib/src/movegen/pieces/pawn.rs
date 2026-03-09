use crate::{
    bitboard::BitBoard,
    game::Game,
    movegen::{
        moves::Move,
        pieces::piece::{PieceColor, PieceMoveInfo, PieceType},
    },
    square::Square,
};

impl Square {
    /// Generates all psuedo legal moves for a single pawn
    /// En_Passant is considered
    /// Promotion is considered (only for queen)
    /// King safety not considered
    pub fn pawn_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();

        let friendly = game.turn;
        let enemy_color = friendly.opponent();

        let initial = match friendly {
            PieceColor::White => BitBoard::INITIAL_WHITE_PAWN,
            PieceColor::Black => BitBoard::INITIAL_BLACK_PAWN,
        };

        let final_rank = friendly.final_rank();

        // Advances
        if let Some(once) = self.forward(&friendly) {
            let oncebb = BitBoard::from_square(once);
            if game.determine_piece(&oncebb).is_none() {
                if once.get_rank() == final_rank {
                    // TODO: Add promotion for pieces other than queen
                    moves.push(Move::Promotion {
                        from: self.get_file(),
                        to: self.get_file(),
                        piece: PieceType::Queen,
                        capture: None,
                    });
                } else {
                    moves.push(Move::Normal {
                        from: *self,
                        to: once,
                        capture: None,
                    });
                }
            }

            // If on initial rank
            if self.in_bitboard(&initial) {
                let twice = once.forward(&friendly).unwrap();
                let twicebb = BitBoard::from_square(twice);
                if game.determine_piece(&twicebb).is_none() {
                    moves.push(Move::CreateEnPassant {
                        at: self.get_file(),
                    });
                }
            }
        }

        // Captures
        // TODO: Add promotion for pieces other than queen
        for diagnol in [self.fleft(&friendly), self.fright(&friendly)]
            .into_iter()
            .flatten()
        {
            let diagnolbb = BitBoard::from_square(diagnol);
            if let Some((piece, enemy)) = game.determine_piece(&diagnolbb) {
                if enemy == enemy_color {
                    if diagnol.get_rank() == final_rank {
                        moves.push(Move::Promotion {
                            from: self.get_file(),
                            to: diagnol.get_file(),
                            piece: PieceType::Queen,
                            capture: Some(piece),
                        });
                    } else {
                        // if piece == PieceType::King {
                        //     let num_checks = game.get_num_checks_mut(&enemy);
                        //     *num_checks += 1;
                        // }

                        moves.push(Move::Normal {
                            from: *self,
                            to: diagnol,
                            capture: Some(piece),
                        });
                    }
                }
            } else if let Some(target) = game.en_passant_target
                && diagnol == target
            {
                moves.push(Move::CaptureEnPassant {
                    from: self.get_file(),
                });
            }
        }

        moves
    }

    pub fn pawn_psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let friendly = game
            .determine_color(&BitBoard::from_square(*self))
            .expect("Tried to move non existent pawn");
        let enemy_color = friendly.opponent();

        let initial = match friendly {
            PieceColor::White => BitBoard::INITIAL_WHITE_PAWN,
            PieceColor::Black => BitBoard::INITIAL_BLACK_PAWN,
        };

        // Advances
        if let Some(once) = self.forward(&friendly) {
            let oncebb = BitBoard::from_square(once);
            if game.determine_piece(&oncebb).is_none() {
                moveinfo.targets |= oncebb;

                // If on initial rank
                if self.in_bitboard(&initial) {
                    let twice = once.forward(&friendly).unwrap();
                    let twicebb = BitBoard::from_square(twice);
                    if game.determine_piece(&twicebb).is_none() {
                        moveinfo.targets |= twicebb;
                    }
                }
            }
        }

        // Captures
        // TODO: Add promotion for pieces other than queen
        for diagnol in [self.fleft(&friendly), self.fright(&friendly)]
            .into_iter()
            .flatten()
        {
            let diagnolbb = BitBoard::from_square(diagnol);
            moveinfo.attacks |= diagnolbb;
            if let Some(enemy) = game.determine_color(&diagnolbb) {
                if enemy == enemy_color {
                    moveinfo.targets |= diagnolbb;
                }
            } else if let Some(target) = game.en_passant_target
                && diagnol == target
            {
                moveinfo.targets |= diagnolbb;
            }
        }

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::File, test_utils::format_pretty_list};

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
        let moves = looking_for
            .from(&game)
            .pawn_psuedo_legal_moves(&mut game);
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
        let moves = looking_for
            .from(&game)
            .pawn_psuedo_legal_moves(&mut game);
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
        let moves = looking_for
            .from(&game)
            .pawn_psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
