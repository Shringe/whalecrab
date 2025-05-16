use crate::{
    bitboard::BitBoard,
    board::{Board, Color},
    movegen::moves::{Move, SpecialMove},
    square::Square,
};

/// Generates all psuedo legal moves for a single pawn
/// En_Passant is considered
/// Promotion not considered
/// King safety not considered
pub fn generate_psuedo_legal_pawn_moves(board: &Board, sq: Square) -> Vec<Move> {
    let mut moves = Vec::new();

    let color = board
        .determine_color(sq)
        .expect("Couldn't determine piece color!");
    let enemy_color = &color.opponent();

    let initial = match color {
        Color::White => BitBoard::INITIAL_WHITE_PAWN,
        Color::Black => BitBoard::INITIAL_BLACK_PAWN,
    };

    // Advances
    if let Some(once) = sq.forward(&color) {
        if board.determine_piece(once).is_none() {
            moves.push(Move {
                from: sq,
                to: once,
                special: None,
            });
        }

        // If on initial rank
        if sq.in_bitboard(&initial) {
            let twice = once.forward(&color).unwrap();
            if board.determine_piece(twice).is_none() {
                moves.push(Move {
                    from: sq,
                    to: twice,
                    special: Some(SpecialMove::CreateEnPassant),
                });
            }
        }
    }

    // Captures
    for diagnol in [sq.fleft(&color), sq.fright(&color)].into_iter().flatten() {
        if let Some(enemy) = board.determine_color(diagnol) {
            if enemy == *enemy_color {
                moves.push(Move {
                    from: sq,
                    to: diagnol,
                    special: None,
                });
            }
        } else if let Some(target) = board.en_passant_target {
            if diagnol == target {
                moves.push(Move {
                    from: sq,
                    to: target,
                    special: Some(SpecialMove::CaptureEnPassant),
                });
            }
        }
    }

    moves
}

/// Generates all available target squares for a pawn. This is primarily used for highlighting
/// playable moves in the TUI
pub fn generate_psuedo_legal_pawn_targets(board: &Board, sq: Square) -> Vec<Square> {
    let moves = generate_psuedo_legal_pawn_moves(board, sq);
    let mut targets = Vec::new();
    for m in moves {
        targets.push(m.to)
    }
    targets
}

/// Generates all moves for all pawn
pub fn generate_all_psuedo_legal_pawn_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    let occupied = match board.turn {
        Color::White => board.white_pawn_bitboard,
        Color::Black => board.black_pawn_bitboard,
    };

    for p in occupied {
        let targets = generate_psuedo_legal_pawn_moves(board, p);
        for t in targets {
            moves.push(t);
        }
    }

    moves
}

// pub fn generate_psuedo_legal_moves(board: Board) -> Vec<Move> {
//
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn psuedo_legal_pawn_moves() {
        let mut board = Board::default();
        let moves = generate_all_psuedo_legal_pawn_moves(&board);

        // White pawn single push
        let expected_move = Move {
            from: Square::C2,
            to: Square::C3,
            special: None,
        };
        assert!(
            moves.contains(&expected_move),
            "Valid white move deemed invalid."
        );

        // White pawn invalid long push
        let not_expected_move = Move {
            from: Square::G2,
            to: Square::G5,
            special: None,
        };
        assert!(
            !moves.contains(&not_expected_move),
            "Invalid white move deemed valid."
        );

        board.turn = board.turn.opponent();
        let moves = generate_all_psuedo_legal_pawn_moves(&board);

        // Black pawn double push from starting rank
        let double_push = Move {
            from: Square::H7,
            to: Square::H5,
            special: Some(SpecialMove::CreateEnPassant),
        };
        assert!(
            moves.contains(&double_push),
            "Valid black double push deemed invalid."
        );

        // Black pawn single push
        let single_push = Move {
            from: Square::E7,
            to: Square::E6,
            special: None,
        };
        assert!(
            moves.contains(&single_push),
            "Valid black single push deemed invalid."
        );

        // Black pawn invalid move (3-step)
        let invalid_black_move = Move {
            from: Square::A7,
            to: Square::A4,
            special: None,
        };
        assert!(
            !moves.contains(&invalid_black_move),
            "Invalid black move deemed valid."
        );
    }

    #[test]
    fn white_pawn_sees_en_passant_target() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::D5,
            to: Square::C6,
            special: Some(SpecialMove::CaptureEnPassant),
        };
        for m in [
            Move {
                from: Square::D2,
                to: Square::D4,
                special: Some(SpecialMove::CreateEnPassant),
            },
            Move {
                from: Square::E7,
                to: Square::E5,
                special: Some(SpecialMove::CreateEnPassant),
            },
            Move {
                from: Square::D4,
                to: Square::D5,
                special: None,
            },
            Move {
                from: Square::C7,
                to: Square::C5,
                special: Some(SpecialMove::CreateEnPassant),
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::White);
        assert!(
            looking_for.from.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );

        assert!(
            board.en_passant_target.is_some(),
            "There is no en_passant_target!"
        );

        let moves = generate_psuedo_legal_pawn_moves(&board, looking_for.from);
        assert!(
            moves.contains(&looking_for),
            "White pawn doesn't see en passant: {}!
Available: {}
board.en_passant_target: {}",
            looking_for,
            format_pretty_list(&moves),
            board.en_passant_target.unwrap()
        );
    }

    #[test]
    fn white_pawn_sees_black_target() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::H4,
            to: Square::G5,
            special: None,
        };
        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                special: None,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                special: None,
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::White);
        assert!(
            looking_for.to.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        let moves = generate_psuedo_legal_pawn_moves(&board, looking_for.from);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn black_pawn_sees_white_target() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::D5,
            to: Square::C4,
            special: None,
        };
        for m in [
            Move {
                from: Square::C2,
                to: Square::C4,
                special: Some(SpecialMove::CreateEnPassant),
            },
            Move {
                from: Square::D7,
                to: Square::D5,
                special: None,
            },
            Move {
                from: Square::H2,
                to: Square::H3,
                special: None,
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::Black);
        assert!(
            looking_for.to.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );
        let moves = generate_psuedo_legal_pawn_moves(&board, looking_for.from);
        assert!(
            moves.contains(&looking_for),
            "Black pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
