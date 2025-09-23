use crate::{
    bitboard::BitBoard,
    board::{Board, PieceType},
    movegen::moves::{get_targets, Move, MoveType},
    square::{Direction, Square},
};

/// Movegeneration helper for ray pieces
pub fn populate_ray_piece(
    from_square: &Square,
    directions: &[Direction],
    board: &mut Board,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let color = board.turn;

    for direction in directions {
        for sq in from_square.ray(direction, board) {
            let attack_bitboard = board.get_occupied_attack_bitboard_mut(&color);
            attack_bitboard.set(sq);
            moves.push(Move {
                from: *from_square,
                to: sq,
                variant: MoveType::Normal,
            });
        }
    }

    moves
}

pub trait Piece {
    /// Generates psuedo legal moves not considering king safety.
    fn psuedo_legal_moves(&self, board: &mut Board) -> Vec<Move>;

    /// Generates psuedo legal targets. Useful for highlighting squares in the TUI.
    fn psuedo_legal_targets(&self, board: &mut Board) -> Vec<Square> {
        let moves = self.psuedo_legal_moves(board);
        get_targets(moves)
    }

    /// Generates legal moves considering king safety.
    fn legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let psuedo_legal = self.psuedo_legal_moves(board);
        let mut legal = Vec::new();

        let attack_board = board.get_occupied_attack_bitboard(&board.turn.opponent());

        for m in psuedo_legal {
            let piece = board
                .determine_piece(m.from)
                .expect("Can't move nonexisting piece!");

            if piece == PieceType::King {
                let kingtobb = BitBoard::from_square(m.to);
                if attack_board.has_square(&kingtobb) {
                    continue;
                }
            }

            legal.push(m);
        }

        legal
    }

    /// Generates legal targets. Useful for highlighting squares in the TUI.
    fn legal_targets(&self, board: &mut Board) -> Vec<Square> {
        let moves = self.legal_moves(board);
        get_targets(moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_cant_blunder_king() {
        let fen = "1k6/1r6/8/8/8/8/8/K7 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();
        let psuedo_legal = board.generate_all_psuedo_legal_moves();
        let legal = board.generate_all_legal_moves();

        let legal_looking_for = vec![Move::new(Square::A1, Square::A2, &board)];
        let psuedo_legal_looking_for = vec![
            Move::new(Square::A1, Square::A2, &board),
            Move::new(Square::A1, Square::B1, &board),
            Move::new(Square::A1, Square::B2, &board),
        ];

        assert_eq!(
            psuedo_legal, psuedo_legal_looking_for,
            "Psuedo_legal moves incorrect"
        );
        assert_ne!(
            psuedo_legal, legal,
            "Illegal psuedo legal moves not filtered out in legal move generation"
        );
        assert_eq!(legal, legal_looking_for, "Legal moves incorrect");
    }

    #[test]
    fn block_check_with_piece() {
        let fen = "4k3/8/8/8/8/8/3r4/4K3 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let blocking_move = Move::new(Square::D2, Square::D1, &board);

        assert!(
            legal_moves.contains(&blocking_move),
            "Rook should be able to block the attacking piece"
        );
    }

    #[test]
    fn must_move_out_of_check() {
        let fen = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();

        // King is in check from e2 rook; only escape squares are d1 and f1
        let expected_moves = vec![
            Move::new(Square::E1, Square::D1, &board),
            Move::new(Square::E1, Square::F1, &board),
        ];

        assert_eq!(
            legal_moves.len(),
            expected_moves.len(),
            "Unexpected number of legal moves"
        );

        for m in expected_moves {
            assert!(
                legal_moves.contains(&m),
                "Expected move {} missing in legal moves",
                m
            );
        }
    }

    #[test]
    fn capture_checking_piece() {
        let fen = "4k3/8/8/8/8/8/4r3/3QK3 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let expected_capture = Move::new(Square::D1, Square::E2, &board);

        assert!(
            legal_moves.contains(&expected_capture),
            "Queen should be able to capture checking rook"
        );
    }

    #[test]
    fn pinned_piece_cannot_move() {
        let fen = "4k3/8/8/8/4r3/8/4P3/4K3 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let pseudo_legal = board.generate_all_psuedo_legal_moves();

        let illegal_due_to_pin = Move::new(Square::E2, Square::E3, &board);

        assert!(
            pseudo_legal.contains(&illegal_due_to_pin),
            "Pinned move should be in pseudo-legal moves"
        );
        assert!(
            !legal_moves.contains(&illegal_due_to_pin),
            "Pinned piece should not be able to move legally"
        );
    }
}
