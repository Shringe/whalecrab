use crate::{
    bitboard::BitBoard,
    game::Game,
    movegen::{moves::Move, pieces::piece::PieceMoveInfo},
    square::Square,
};

impl Square {
    pub fn knight_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();
        let rank = self.get_rank();
        let file = self.get_file();

        let friendly = game.turn;

        let mut process_target = |t: Square| {
            let tbb = BitBoard::from_square(t);

            if let Some((piece, color)) = game.determine_piece(&tbb) {
                if color == friendly {
                    return;
                }

                moves.push(Move::Normal {
                    from: *self,
                    to: t,
                    capture: Some(piece),
                });
            } else {
                moves.push(Move::Normal {
                    from: *self,
                    to: t,
                    capture: None,
                });
            }
        };

        if rank.to_index() < 6 {
            let north = Square::make_square(rank.up().up(), file);
            for t in [north.left(), north.right()].into_iter().flatten() {
                process_target(t);
            }
        }

        if rank.to_index() > 1 {
            let south = Square::make_square(rank.down().down(), file);
            for t in [south.left(), south.right()].into_iter().flatten() {
                process_target(t);
            }
        }

        if file.to_index() < 6 {
            let east = Square::make_square(rank, file.right().right());
            for t in [east.up(), east.down()].into_iter().flatten() {
                process_target(t);
            }
        }

        if file.to_index() > 1 {
            let west = Square::make_square(rank, file.left().left());
            for t in [west.up(), west.down()].into_iter().flatten() {
                process_target(t);
            }
        }

        moves
    }

    pub fn knight_psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let rank = self.get_rank();
        let file = self.get_file();

        let enemy = game.turn.opponent();

        let mut process_target = |t: Square| {
            let tbb = BitBoard::from_square(t);
            moveinfo.attacks |= tbb;

            if let Some(color) = game.determine_color(&tbb) {
                if color == enemy {
                    moveinfo.targets |= tbb;
                }
            } else {
                moveinfo.targets |= tbb;
            }
        };

        if rank.to_index() < 6 {
            let north = Square::make_square(rank.up().up(), file);
            for t in [north.left(), north.right()].into_iter().flatten() {
                process_target(t);
            }
        }

        if rank.to_index() > 1 {
            let south = Square::make_square(rank.down().down(), file);
            for t in [south.left(), south.right()].into_iter().flatten() {
                process_target(t);
            }
        }

        if file.to_index() < 6 {
            let east = Square::make_square(rank, file.right().right());
            for t in [east.up(), east.down()].into_iter().flatten() {
                process_target(t);
            }
        }

        if file.to_index() > 1 {
            let west = Square::make_square(rank, file.left().left());
            for t in [west.up(), west.down()].into_iter().flatten() {
                process_target(t);
            }
        }

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::movegen::pieces::piece::{PieceColor, PieceType};
    use crate::test_utils::format_pretty_list;

    use super::*;

    #[test]
    fn knight_cant_capture_en_passant() {
        let mut game = Game::default();
        let avoid = Move::CaptureEnPassant {
            from: Square::E5.get_file(),
        };
        for m in [
            Move::Normal {
                from: Square::G1,
                to: Square::F3,
                capture: None,
            },
            Move::Normal {
                from: Square::A7,
                to: Square::A5,
                capture: None,
            },
            Move::Normal {
                from: Square::F3,
                to: Square::E5,
                capture: None,
            },
            Move::CreateEnPassant {
                at: Square::C7.get_file(),
            },
        ] {
            game.play(&m);
        }

        let moves = avoid
            .from(&game)
            .knight_psuedo_legal_moves(&mut game);
        assert!(!moves.contains(&avoid));
    }

    #[test]
    fn white_knight_captures_black_pawn() {
        let mut game = Game::default();
        let capture = dbg!(Move::Normal {
            from: Square::F5,
            to: Square::E7,
            capture: Some(PieceType::Pawn),
        });

        for m in [
            Move::Normal {
                from: Square::G1,
                to: Square::F3,
                capture: None,
            },
            Move::Normal {
                from: Square::A7,
                to: Square::A6,
                capture: None,
            },
            Move::Normal {
                from: Square::F3,
                to: Square::D4,
                capture: None,
            },
            Move::Normal {
                from: Square::A6,
                to: Square::A5,
                capture: None,
            },
            Move::Normal {
                from: Square::D4,
                to: Square::F5,
                capture: None,
            },
            Move::Normal {
                from: Square::A5,
                to: Square::A4,
                capture: None,
            },
        ] {
            if game.turn == PieceColor::White {
                let moves = m.from(&game).knight_psuedo_legal_moves(&mut game);
                assert!(
                    moves.contains(&m),
                    "Tried to make '{}' in order to set up the board, but it couldn't happen normally! The knight only sees: {}.",
                    m,
                    format_pretty_list(&moves)
                )
            }
            game.play(&m);
        }

        let moves = capture
            .from(&game)
            .knight_psuedo_legal_moves(&mut game);

        assert!(
            moves.contains(&capture),
            "Knight did not generate expected capture move. Availabe: {}",
            format_pretty_list(&moves)
        );

        let knight_before = game.white_knights.popcnt();
        let pawns_before = game.black_pawns.popcnt();
        game.play(&capture);
        let knight_after = game.white_knights.popcnt();
        let pawns_after = game.black_pawns.popcnt();

        assert_eq!(knight_before, knight_after, "We lost the knight!");
        assert_eq!(
            pawns_before - 1,
            pawns_after,
            "The pawn is still standing knight!"
        );
    }
}
