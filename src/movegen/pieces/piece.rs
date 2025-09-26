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
        let attack_board = board.get_attacks(&color.opponent());
        let check_ray_board = board.get_check_rays(&color.opponent());

        for m in psuedo_legal {
            let piece = board
                .determine_piece(m.from)
                .expect("Can't move nonexisting piece!");
            let frombb = BitBoard::from_square(m.from);
            let tobb = BitBoard::from_square(m.to);

            let num_checks = board.get_num_checks(color);
            let is_moving_king = piece == PieceType::King;
            let is_capturing = m.get_capture(&board).is_some();
            let is_blocking = board.get_check_rays(&color.opponent()) & tobb != EMPTY;

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

            if is_moving_king {
                // Prevent moving into check
                if attack_board.has_square(&tobb) {
                    continue;
                }
            } else {
                // Prevent moving piece blocking check (pin)
                if check_ray_board.has_square(&frombb) {
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
    use std::collections::HashMap;

    use crate::test_utils::{format_pretty_list, should_generate, shouldnt_generate};

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

    fn ensure_legal_game(mut board: Board, game_turns: &Vec<(Square, Square)>) {
        let mut move_num = 0;
        let mut psuedo_illegal_moves = HashMap::new();
        let mut illegal_moves = HashMap::new();
        for (i, to_play) in game_turns.iter().enumerate() {
            let to_play = Move::new(to_play.0, to_play.1, &board);
            let fen = board.to_fen();
            let psuedo_legal_moves = board.generate_all_psuedo_legal_moves();
            let legal_moves = board.generate_all_legal_moves();

            let turn = i + 1;
            if i % 2 == 0 {
                move_num += 1;
            }

            if !psuedo_legal_moves.contains(&to_play) {
                let short = format!(
                    "Move: {}, Turn: {}. The move {} was deemed psuedo illegal\n  {}",
                    move_num, turn, to_play, fen
                );

                let long = format!(
                    "Available moves: {}",
                    format_pretty_list(&psuedo_legal_moves)
                );

                psuedo_illegal_moves.insert(short, long);
            }

            let color = board.turn;
            let piece = if let Some(piece) = board.determine_piece(to_play.from) {
                piece
            } else {
                let short = format!(
                    "Move: {}, Turn: {}. Tried to move nonexistant piece at square: {}\n  {}",
                    move_num, turn, to_play.from, fen
                );
                let long = short.clone();
                psuedo_illegal_moves.insert(short, long);
                break;
            };
            let piece_attacks = BitBoard::from_square_vec(get_targets(
                piece.get_psuedo_legal_moves(&mut board, to_play.from),
            ));

            let piece_attacks_legal = BitBoard::from_square_vec(get_targets(
                piece.get_legal_moves(&mut board, to_play.from),
            ));

            if !legal_moves.contains(&to_play) {
                let short = format!(
                    "Move: {}, Turn: {}. The move {} was deemed illegal\n  {}",
                    move_num, turn, to_play, fen
                );

                let long = format!(
                    "Piece info:
type: {:?}
color: {:?}
location: {}
wants: {}
psuedo legally attacking:
{}

legally attacking:
{}

White Board info:
num_checks: {}
ray_attacks:
{}

attacks:
{}

Black Board info:
num_checks: {}
ray_attacks:
{}

attacks:
{}

Available moves: {}
",
                    piece,
                    color,
                    to_play.from,
                    to_play.to,
                    piece_attacks,
                    piece_attacks_legal,
                    board.white_num_checks,
                    board.white_check_rays,
                    board.white_attacks,
                    board.black_num_checks,
                    board.black_check_rays,
                    board.black_attacks,
                    format_pretty_list(&legal_moves)
                );

                illegal_moves.insert(short, long);
            }

            board = to_play.make(&board);
        }

        match psuedo_illegal_moves.len() {
            0 => {}
            1 => {
                let (short, long) = psuedo_illegal_moves.iter().next().unwrap();
                panic!("{}\n{}", short, long);
            }
            _ => {
                for (short, _) in &psuedo_illegal_moves {
                    println!("{}", short);
                }
                panic!(
                    "{} psuedo illegal moves were found",
                    psuedo_illegal_moves.len()
                );
            }
        }

        match illegal_moves.len() {
            0 => {}
            1 => {
                let (short, long) = illegal_moves.iter().next().unwrap();
                panic!("{}\n{}", short, long);
            }
            _ => {
                for (short, _) in &illegal_moves {
                    println!("{}", short);
                }
                panic!("{} illegal moves were found", illegal_moves.len());
            }
        }
    }

    /// https://www.chessgames.com/perl/chessgame?gid=1242968
    #[test]
    fn queens_gambit_game() {
        let board = Board::default();
        let game_turns = vec![
            (Square::D2, Square::D4),
            (Square::D7, Square::D5),
            (Square::C2, Square::C4),
            (Square::E7, Square::E6),
            (Square::B1, Square::C3),
            (Square::G8, Square::F6),
            (Square::C1, Square::G5),
            (Square::B8, Square::D7),
            (Square::C4, Square::D5),
            (Square::E6, Square::D5),
            (Square::C3, Square::D5),
            (Square::F6, Square::D5),
            (Square::G5, Square::D8),
            (Square::F8, Square::B4),
            (Square::D1, Square::D2),
            (Square::E8, Square::D8),
        ];

        ensure_legal_game(board, &game_turns);
    }

    /// https://www.chessgames.com/perl/chessgame?gid=1955216
    #[test]
    fn sicilian_defense_game() {
        let board = Board::default();
        let game_turns = vec![
            (Square::E2, Square::E4),
            (Square::C7, Square::C5),
            (Square::G1, Square::F3),
            (Square::B8, Square::C6),
            (Square::B1, Square::C3),
            (Square::E7, Square::E5),
            (Square::F1, Square::C4),
            (Square::F8, Square::E7),
            (Square::D2, Square::D3),
            (Square::D7, Square::D6),
            (Square::F3, Square::D2),
            (Square::G8, Square::F6),
            (Square::D2, Square::F1),
            (Square::F6, Square::D7),
            (Square::C3, Square::D5),
            (Square::D7, Square::B6),
            (Square::D5, Square::B6),
            (Square::A7, Square::B6),
            (Square::C2, Square::C3),
            (Square::E8, Square::G8),
            (Square::F1, Square::E3),
            (Square::E7, Square::G5),
            (Square::E1, Square::G1),
            (Square::G8, Square::H8),
            (Square::A2, Square::A3),
            (Square::F7, Square::F5),
            (Square::E3, Square::F5),
            (Square::G5, Square::C1),
            (Square::A1, Square::C1),
            (Square::C8, Square::F5),
            (Square::E4, Square::F5),
            (Square::D6, Square::D5),
            (Square::C4, Square::A2),
            (Square::F8, Square::F5),
            (Square::D1, Square::G4),
            (Square::F5, Square::F6),
            (Square::F2, Square::F4),
            (Square::E5, Square::F4),
            (Square::G4, Square::G5),
            (Square::D8, Square::F8),
            (Square::G5, Square::D5),
            (Square::A8, Square::D8),
            (Square::D5, Square::F3),
            (Square::C6, Square::E5),
            (Square::F3, Square::E4),
            (Square::E5, Square::G4),
            (Square::C1, Square::E1),
            (Square::G4, Square::E3),
            (Square::F1, Square::F2),
            (Square::D8, Square::E8),
            (Square::E4, Square::B7),
            (Square::G7, Square::G5),
            (Square::F2, Square::E2),
            (Square::G5, Square::G4),
            (Square::E2, Square::F2),
            (Square::F8, Square::H6),
            (Square::B7, Square::C7),
            (Square::E8, Square::F8),
            (Square::H2, Square::H3),
            (Square::G4, Square::H3),
            (Square::G2, Square::G3),
            (Square::F4, Square::G3),
            (Square::F2, Square::F6),
            (Square::H3, Square::H2),
            (Square::G1, Square::H1),
            (Square::G3, Square::G2),
        ];

        ensure_legal_game(board, &game_turns);
    }
}
