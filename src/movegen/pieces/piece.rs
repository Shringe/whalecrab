use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{Board, PieceType},
    movegen::moves::{get_targets, Move},
    square::Square,
};

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

        let color = &board.turn;
        let attack_board = board.get_occupied_attack_bitboard(&color.opponent());

        for m in psuedo_legal {
            let piece = board
                .determine_piece(m.from)
                .expect("Can't move nonexisting piece!");
            // let frombb = BitBoard::from_square(m.from);
            let tobb = BitBoard::from_square(m.to);

            let num_checks = board.get_num_checks(color);
            let is_moving_king = piece == PieceType::King;
            let is_capturing = m.get_capture(&board).is_some();
            let is_blocking =
                board.get_occupied_attack_ray_bitboard(&color.opponent()) & tobb != EMPTY;

            // Handle being in check
            match *num_checks {
                1 => {
                    if !(is_moving_king || is_capturing || is_blocking) {
                        continue;
                    }
                }
                2 => {
                    if !is_moving_king {
                        continue;
                    }
                }
                _ => {}
            }

            // prevent moving into check
            if is_moving_king && attack_board.has_square(&tobb) {
                continue;
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
    use crate::test_utils::{should_generate, shouldnt_generate};

    use super::*;

    #[test]
    fn cant_move_into_check() {
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
        let fen = "4k3/4r3/8/8/2N5/8/4K3/8 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let looking_for = Move::new(Square::C4, Square::E3, &board);

        should_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn must_move_out_of_check() {
        let fen = "4k3/4r3/8/8/8/3P1P2/4KP2/3RRR2 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let looking_for = [Move::new(Square::E2, Square::D2, &board)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn capture_checking_piece() {
        let fen = "4k3/4r3/8/8/1B6/3P1P2/3PKP2/3RRR2 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let looking_for = [Move::new(Square::B4, Square::E7, &board)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn pinned_piece_cannot_move() {
        let fen = "4k3/4r3/8/8/3P1P2/4B3/3PK3/6P1 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let looking_for = Move::new(Square::E3, Square::F2, &board);

        shouldnt_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn cant_move_king_within_check_ray() {
        let fen = "4K3/4R3/8/8/8/8/4k3/8 b - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();

        let legal_moves = board.generate_all_legal_moves();
        let looking_for = [
            Move::new(Square::E2, Square::E1, &board),
            Move::new(Square::E2, Square::E3, &board),
        ];

        for m in looking_for {
            shouldnt_generate(&legal_moves, &m);
        }
    }

    #[test]
    fn must_move_out_of_double_check() {
        let fen = "4k3/4r3/8/6Qb/8/2R5/4KP2/8 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        board.initialize();
        let legal_moves = board.generate_all_legal_moves();
        let king = Square::E2;

        for m in legal_moves {
            assert_eq!(
                m.from, king,
                "Tried to move piece other than queen while double check. {}",
                m
            );
        }
    }
}
