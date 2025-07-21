use crate::{
    board::{Board, Color},
    movegen::moves::Move,
};

use super::pieces::{pawn::Pawn, piece::Piece};

/// Generates all moves for all pawn
pub fn generate_all_psuedo_legal_pawn_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    let occupied = match board.turn {
        Color::White => board.white_pawn_bitboard,
        Color::Black => board.black_pawn_bitboard,
    };

    for p in occupied {
        let targets = Pawn(p).psuedo_legal_moves(board);
        for t in targets {
            moves.push(t);
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{movegen::moves::MoveType, square::Square, test_utils::*};

    #[test]
    fn psuedo_legal_pawn_moves() {
        let mut board = Board::default();
        let moves = generate_all_psuedo_legal_pawn_moves(&board);

        // White pawn single push
        let expected_move = Move {
            from: Square::C2,
            to: Square::C3,
            variant: MoveType::Normal,
        };
        assert!(
            moves.contains(&expected_move),
            "Valid white move deemed invalid."
        );

        // White pawn invalid long push
        let not_expected_move = Move {
            from: Square::G2,
            to: Square::G5,
            variant: MoveType::Normal,
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
            variant: MoveType::CreateEnPassant,
        };
        assert!(
            moves.contains(&double_push),
            "Valid black double push deemed invalid."
        );

        // Black pawn single push
        let single_push = Move {
            from: Square::E7,
            to: Square::E6,
            variant: MoveType::Normal,
        };
        assert!(
            moves.contains(&single_push),
            "Valid black single push deemed invalid."
        );

        // Black pawn invalid move (3-step)
        let invalid_black_move = Move {
            from: Square::A7,
            to: Square::A4,
            variant: MoveType::Normal,
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
            variant: MoveType::CaptureEnPassant,
        };
        for m in [
            Move {
                from: Square::D2,
                to: Square::D4,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::E7,
                to: Square::E5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::D4,
                to: Square::D5,
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

        assert_eq!(board.turn, Color::White);
        assert!(
            looking_for.from.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );

        assert!(
            board.en_passant_target.is_some(),
            "There is no en_passant_target!"
        );

        let moves = Pawn(looking_for.from).psuedo_legal_moves(&board);
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
}
