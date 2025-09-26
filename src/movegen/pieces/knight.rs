use crate::{
    board::{Board, PieceType},
    movegen::moves::{Move, MoveType},
    square::Square,
};

use super::piece::Piece;

pub struct Knight(pub Square);

impl Piece for Knight {
    fn psuedo_legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let rank = self.0.get_rank();
        let file = self.0.get_file();

        let color = board.turn;
        let enemy = color.opponent();
        let friendly = Some(color);

        if rank.to_index() < 5 {
            let north = Square::make_square(rank.up().up(), file);
            for t in [north.left(), north.right()].into_iter().flatten() {
                let attack_bitboard = board.get_attacks_mut(&color);
                attack_bitboard.set(t);

                if board.determine_color(t) == friendly {
                    continue;
                }

                if board.determine_piece(t) == Some(PieceType::King) {
                    let num_checks = board.get_num_checks_mut(&enemy);
                    *num_checks += 1;
                }

                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
                });
            }
        }

        if rank.to_index() > 1 {
            let south = Square::make_square(rank.down().down(), file);
            for t in [south.left(), south.right()].into_iter().flatten() {
                let attack_bitboard = board.get_attacks_mut(&color);
                attack_bitboard.set(t);

                if board.determine_color(t) == friendly {
                    continue;
                }

                if board.determine_piece(t) == Some(PieceType::King) {
                    let num_checks = board.get_num_checks_mut(&enemy);
                    *num_checks += 1;
                }

                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
                });
            }
        }

        if file.to_index() < 5 {
            let east = Square::make_square(rank, file.right().right());
            for t in [east.up(), east.down()].into_iter().flatten() {
                let attack_bitboard = board.get_attacks_mut(&color);
                attack_bitboard.set(t);

                if board.determine_color(t) == friendly {
                    continue;
                }

                if board.determine_piece(t) == Some(PieceType::King) {
                    let num_checks = board.get_num_checks_mut(&enemy);
                    *num_checks += 1;
                }

                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
                });
            }
        }

        if file.to_index() > 1 {
            let west = Square::make_square(rank, file.left().left());
            for t in [west.up(), west.down()].into_iter().flatten() {
                let attack_bitboard = board.get_attacks_mut(&color);
                attack_bitboard.set(t);

                if board.determine_color(t) == friendly {
                    continue;
                }

                if board.determine_piece(t) == Some(PieceType::King) {
                    let num_checks = board.get_num_checks_mut(&enemy);
                    *num_checks += 1;
                }

                moves.push(Move {
                    from: self.0,
                    to: t,
                    variant: MoveType::Normal,
                });
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Color, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn knight_cant_capture_en_passant() {
        let mut board = Board::default();
        let avoid = Move {
            from: Square::E5,
            to: Square::C6,
            variant: MoveType::CaptureEnPassant,
        };
        for m in [
            Move {
                from: Square::G1,
                to: Square::F3,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::A7,
                to: Square::A5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::F3,
                to: Square::E5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::C7,
                to: Square::C5,
                variant: MoveType::CreateEnPassant,
            },
        ] {
            board = m.make(&board);
        }

        let moves = Knight(avoid.from).psuedo_legal_moves(&mut board);
        assert!(!moves.contains(&avoid));
    }

    #[test]
    fn white_knight_captures_black_pawn() {
        let mut board = Board::default();
        let capture = Move {
            from: Square::F5,
            to: Square::E7,
            variant: MoveType::Normal,
        };

        for m in [
            Move {
                from: Square::G1,
                to: Square::F3,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::A7,
                to: Square::A6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::F3,
                to: Square::D4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::A6,
                to: Square::A5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::D4,
                to: Square::F5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::A5,
                to: Square::A4,
                variant: MoveType::Normal,
            },
        ] {
            if board.turn == Color::White {
                let moves = Knight(m.from).psuedo_legal_moves(&mut board);
                assert!(
                    moves.contains(&m),
                    "Tried to make '{}' in order to set up the board, but it couldn't happen normally! The knight only sees: {}.",
                    m,
                    format_pretty_list(&moves)
                )
            }
            board = m.make(&board);
        }

        let moves = Knight(capture.from).psuedo_legal_moves(&mut board);

        assert!(
            moves.contains(&capture),
            "Knight did not generate expected capture move. Availabe: {}",
            format_pretty_list(&moves)
        );

        let knight_before = board.white_knights.popcnt();
        let pawns_before = board.black_pawns.popcnt();
        let board = capture.make(&board);
        let knight_after = board.white_knights.popcnt();
        let pawns_after = board.black_pawns.popcnt();

        assert_eq!(knight_before, knight_after, "We lost the knight!");
        assert_eq!(
            pawns_before - 1,
            pawns_after,
            "The pawn is still standing knight!"
        );
    }
}
