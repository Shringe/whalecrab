use crate::{
    bitboard::BitBoard,
    board::{Color, PieceType},
    game::Game,
    movegen::moves::{Move, MoveType},
    square::Square,
};

use super::piece::Piece;

pub struct Pawn(pub Square);

impl Piece for Pawn {
    /// Generates all psuedo legal moves for a single pawn
    /// En_Passant is considered
    /// Promotion is considered (only for queen)
    /// King safety not considered
    fn psuedo_legal_moves(&self, game: &mut Game) -> Vec<Move> {
        let mut moves = Vec::new();

        let friendly = game.position.turn;
        let enemy_color = friendly.opponent();

        let initial = match friendly {
            Color::White => BitBoard::INITIAL_WHITE_PAWN,
            Color::Black => BitBoard::INITIAL_BLACK_PAWN,
        };

        let final_rank = friendly.final_rank();

        // Advances
        if let Some(once) = self.0.forward(&friendly) {
            let oncebb = BitBoard::from_square(once);
            if game.determine_piece(&oncebb).is_none() {
                if once.get_rank() == final_rank {
                    // TODO: Add promotion for pieces other than queen
                    moves.push(Move {
                        from: self.0,
                        to: once,
                        variant: MoveType::Promotion(PieceType::Queen),
                    });
                } else {
                    moves.push(Move {
                        from: self.0,
                        to: once,
                        variant: MoveType::Normal,
                    });
                }
            }

            // If on initial rank
            if self.0.in_bitboard(&initial) {
                let twice = once.forward(&friendly).unwrap();
                let twicebb = BitBoard::from_square(twice);
                if game.determine_piece(&twicebb).is_none() {
                    moves.push(Move {
                        from: self.0,
                        to: twice,
                        variant: MoveType::CreateEnPassant,
                    });
                }
            }
        }

        // Captures
        // TODO: Add promotion for pieces other than queen
        for diagnol in [self.0.fleft(&friendly), self.0.fright(&friendly)]
            .into_iter()
            .flatten()
        {
            let diagnolbb = BitBoard::from_square(diagnol);
            let attack_bitboard = game.get_attacks_mut(&friendly);
            attack_bitboard.set(diagnol);
            if let Some((piece, enemy)) = game.determine_piece(&diagnolbb) {
                if enemy == enemy_color {
                    if diagnol.get_rank() == final_rank {
                        moves.push(Move {
                            from: self.0,
                            to: diagnol,
                            variant: MoveType::Promotion(PieceType::Queen),
                        });
                    } else {
                        if piece == PieceType::King {
                            let num_checks = game.get_num_checks_mut(&enemy);
                            *num_checks += 1;
                        }

                        moves.push(Move {
                            from: self.0,
                            to: diagnol,
                            variant: MoveType::Normal,
                        });
                    }
                }
            } else if let Some(target) = game.position.en_passant_target {
                if diagnol == target {
                    moves.push(Move {
                        from: self.0,
                        to: target,
                        variant: MoveType::CaptureEnPassant,
                    });
                }
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::format_pretty_list;

    use super::*;

    #[test]
    fn white_pawn_sees_black_target() {
        let mut game = Game::default();
        let looking_for = Move {
            from: Square::H4,
            to: Square::G5,
            variant: MoveType::Normal,
        };
        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                variant: MoveType::Normal,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.position.turn, Color::White);
        assert!(
            looking_for.to.in_bitboard(&game.position.black_pawns),
            "Black pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&game.position.white_pawns),
            "White pawn not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn black_pawn_sees_white_target() {
        let mut game = Game::default();
        let looking_for = Move {
            from: Square::D5,
            to: Square::C4,
            variant: MoveType::Normal,
        };

        for m in [
            Move {
                from: Square::C2,
                to: Square::C4,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::D7,
                to: Square::D5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H2,
                to: Square::H3,
                variant: MoveType::Normal,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.position.turn, Color::Black);
        assert!(
            looking_for.to.in_bitboard(&game.position.white_pawns),
            "White pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&game.position.black_pawns),
            "Black pawn not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&looking_for),
            "Black pawn can't see target. Available moves: {:?}",
            moves
        );
    }

    #[test]
    fn white_pawn_sees_queen_promotion() {
        let mut game = Game::default();
        let looking_for = Move {
            from: Square::G7,
            to: Square::H8,
            variant: MoveType::Promotion(PieceType::Queen),
        };

        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::H4,
                to: Square::G5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H7,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::G5,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::F8,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H6,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::E7,
                to: Square::E5,
                variant: MoveType::CreateEnPassant,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.position.turn, Color::White);
        assert!(
            looking_for.from.in_bitboard(&game.position.white_pawns),
            "White pawn not in position"
        );
        assert!(
            looking_for.to.in_bitboard(&game.position.black_rooks),
            "Black rook not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
