use crate::{
    bitboard::BitBoard,
    game::Game,
    movegen::{
        moves::{Move, MoveType},
        pieces::piece::PieceMoveInfo,
    },
    square::Square,
};

use super::piece::Piece;

pub struct Knight(pub Square);

impl Piece for Knight {
    fn psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();
        let rank = self.0.get_rank();
        let file = self.0.get_file();

        let friendly = game.position.turn;

        let mut process_target = |t: Square| {
            let tbb = BitBoard::from_square(t);

            if let Some((_piece, color)) = game.determine_piece(&tbb) {
                if color == friendly {
                    return;
                }

                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
                    capture: None,
                });
            } else {
                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
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

    fn psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let rank = self.0.get_rank();
        let file = self.0.get_file();

        let enemy = game.position.turn.opponent();

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
    use crate::movegen::pieces::piece::{Color, PieceType};
    use crate::test_utils::format_pretty_list;

    use super::*;

    #[test]
    fn knight_cant_capture_en_passant() {
        let mut game = Game::default();
        let avoid = Move {
            from: Square::E5,
            to: Square::C6,
            variant: MoveType::CaptureEnPassant,
            capture: None,
        };
        for m in [
            Move {
                from: Square::G1,
                to: Square::F3,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::A7,
                to: Square::A5,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::F3,
                to: Square::E5,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::C7,
                to: Square::C5,
                variant: MoveType::CreateEnPassant,
                capture: None,
            },
        ] {
            game.play(&m);
        }

        let moves = Knight(avoid.from).psuedo_legal_moves(&mut game);
        assert!(!moves.contains(&avoid));
    }

    #[test]
    fn white_knight_captures_black_pawn() {
        let mut game = Game::default();
        let capture = Move {
            from: Square::F5,
            to: Square::E7,
            variant: MoveType::Normal,
            capture: Some(PieceType::Pawn),
        };

        for m in [
            Move {
                from: Square::G1,
                to: Square::F3,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::A7,
                to: Square::A6,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::F3,
                to: Square::D4,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::A6,
                to: Square::A5,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::D4,
                to: Square::F5,
                variant: MoveType::Normal,
                capture: None,
            },
            Move {
                from: Square::A5,
                to: Square::A4,
                variant: MoveType::Normal,
                capture: None,
            },
        ] {
            if game.position.turn == Color::White {
                let moves = Knight(m.from).psuedo_legal_moves(&mut game);
                assert!(
                    moves.contains(&m),
                    "Tried to make '{}' in order to set up the board, but it couldn't happen normally! The knight only sees: {}.",
                    m,
                    format_pretty_list(&moves)
                )
            }
            game.play(&m);
        }

        let moves = Knight(capture.from).psuedo_legal_moves(&mut game);

        assert!(
            moves.contains(&capture),
            "Knight did not generate expected capture move. Availabe: {}",
            format_pretty_list(&moves)
        );

        let knight_before = game.position.white_knights.popcnt();
        let pawns_before = game.position.black_pawns.popcnt();
        game.play(&capture);
        let knight_after = game.position.white_knights.popcnt();
        let pawns_after = game.position.black_pawns.popcnt();

        assert_eq!(knight_before, knight_after, "We lost the knight!");
        assert_eq!(
            pawns_before - 1,
            pawns_after,
            "The pawn is still standing knight!"
        );
    }
}
