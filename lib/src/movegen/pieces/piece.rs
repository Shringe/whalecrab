use crate::{
    bitboard::{BitBoard, EMPTY},
    movegen::moves::Move,
    position::game::Game,
    rank::Rank,
    square::Square,
};

#[derive(Debug, PartialEq, Clone, Hash, Copy)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub const fn from_int(value: u8) -> Option<PieceColor> {
        match value {
            0 => Some(PieceColor::White),
            1 => Some(PieceColor::Black),
            _ => None,
        }
    }

    pub const fn to_int(&self) -> u8 {
        match self {
            PieceColor::White => 0,
            PieceColor::Black => 1,
        }
    }

    pub fn opponent(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }

    pub fn final_rank(&self) -> Rank {
        match self {
            PieceColor::White => Rank::Eighth,
            PieceColor::Black => Rank::First,
        }
    }
}

pub const ALL_PIECE_TYPES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King,
];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub const fn from_int(value: u8) -> Option<PieceType> {
        match value {
            0 => Some(PieceType::Pawn),
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            5 => Some(PieceType::King),
            _ => None,
        }
    }

    pub const fn to_int(&self) -> u8 {
        match self {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5,
        }
    }

    pub fn psuedo_legal_moves(&self, game: &Game, square: &Square) -> Vec<Move> {
        match self {
            PieceType::Pawn => square.pawn_psuedo_legal_moves(game),
            PieceType::Knight => square.knight_psuedo_legal_moves(game),
            PieceType::Bishop => square.bishop_psuedo_legal_moves(game),
            PieceType::Rook => square.rook_psuedo_legal_moves(game),
            PieceType::Queen => square.queen_psuedo_legal_moves(game),
            PieceType::King => square.king_psuedo_legal_moves(game),
        }
    }

    pub fn psuedo_legal_targets_fast(&self, game: &Game, square: &Square) -> PieceMoveInfo {
        match self {
            PieceType::Pawn => square.pawn_psuedo_legal_targets(game),
            PieceType::Knight => square.knight_psuedo_legal_targets(game),
            PieceType::Bishop => square.bishop_psuedo_legal_targets(game),
            PieceType::Rook => square.rook_psuedo_legal_targets(game),
            PieceType::Queen => square.queen_psuedo_legal_targets(game),
            PieceType::King => square.king_psuedo_legal_targets(game),
        }
    }

    pub fn legal_moves(&self, game: &Game, square: &Square) -> Vec<Move> {
        game.legal_moves_filter(self.psuedo_legal_moves(game, square))
    }

    pub fn is_ray_piece(&self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }
}

/// Stores where a piece could move to and what squares it currently defends
#[derive(Default, PartialEq, Eq, Debug)]
pub struct PieceMoveInfo {
    /// The possible squares a piece can move to
    pub targets: BitBoard,
    /// The squares a piece attacks/defends
    pub attacks: BitBoard,
    /// Pins for ray pieces. Empty if not a ray piece
    pub check_rays: BitBoard,
}

/// Gen a bitboard containing every square set between two square indexes, non inclusive
fn between_two_squares(from: Square, to: Square) -> BitBoard {
    let mut out = EMPTY;

    for sq in !EMPTY {
        if sq.to_int() > from.to_int() && sq.to_int() < to.to_int() {
            out |= BitBoard::from_square(sq);
        }
    }

    out
}

impl Game {
    /// Filters out psuedo_legal moves that are found to be illegal
    pub fn legal_moves_filter(&self, psuedo_legal: Vec<Move>) -> Vec<Move> {
        let mut legal = Vec::with_capacity(psuedo_legal.len());

        let enemy = self.turn.opponent();
        let attack_board = self.get_attacks(&enemy);
        let checks = *self.get_check_rays(&enemy);

        let kingbb = self.get_pieces(&PieceType::King, &self.turn);
        let king = kingbb.to_square();
        let king_attackers = self.attackers(king);

        for m in psuedo_legal {
            let from = m.from(self.turn);
            let to = m.to(self);
            let frombb = BitBoard::from_square(from);
            let tobb = BitBoard::from_square(to);

            let is_moving_king = kingbb.has_square(frombb);

            // Handle being in check
            match king_attackers.popcnt() {
                1 => {
                    let is_blocking = (between_two_squares(king, king_attackers.to_square())
                        & checks)
                        .has_square(tobb);
                    let is_capturing = m.is_capture();
                    let is_capturing_attacking_piece =
                        is_capturing && king_attackers.has_square(tobb);

                    if !(is_moving_king || is_capturing_attacking_piece || is_blocking) {
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
                if attack_board.has_square(tobb) {
                    continue;
                }
            } else {
                // Prevent moving piece blocking check (pin)
                if checks.has_square(frombb) {
                    continue;
                }
            }

            debug_assert!(
                !matches!(
                    m,
                    Move::Normal {
                        capture: Some(PieceType::King),
                        ..
                    }
                ),
                "The king is being captured! {}",
                m
            );

            legal.push(m);
        }

        debug_assert!(
            self.white_kings != EMPTY && self.black_kings != EMPTY,
            "There is no king! {:#?} {:?}",
            legal,
            self
        );

        // TODO: remove
        legal.shrink_to_fit();
        legal
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        movegen::moves::moves_to_targets_vec,
        position::game::State,
        square::Square,
        test_utils::{format_pretty_list, should_generate, shouldnt_generate},
    };

    use super::*;

    #[test]
    fn cant_move_into_check() {
        let fen = "1k6/1r6/8/8/8/8/8/K7 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let psuedo_legal = game.generate_all_psuedo_legal_moves();
        let legal = game.legal_moves();

        let legal_looking_for = vec![Move::infer(Square::A1, Square::A2, &game)];
        let psuedo_legal_looking_for = vec![
            Move::infer(Square::A1, Square::B1, &game),
            Move::infer(Square::A1, Square::A2, &game),
            Move::infer(Square::A1, Square::B2, &game),
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
        let mut game = Game::from_fen(fen).unwrap();

        let legal_moves = game.legal_moves();
        let looking_for = Move::infer(Square::C4, Square::E3, &game);

        should_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn must_move_out_of_check() {
        let fen = "4k3/4r3/8/8/8/3P1P2/4KP2/3RRR2 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let legal_moves = game.legal_moves();
        let looking_for = [Move::infer(Square::E2, Square::D2, &game)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn capture_checking_piece() {
        let fen = "4k3/4r3/8/8/1B6/3P1P2/3PKP2/3RRR2 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let legal_moves = game.legal_moves();
        let looking_for = [Move::infer(Square::B4, Square::E7, &game)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn pinned_piece_cannot_move() {
        let fen = "4k3/4r3/8/8/3P1P2/4B3/3PK3/6P1 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let legal_moves = game.legal_moves();
        let looking_for = Move::infer(Square::E3, Square::F2, &game);

        shouldnt_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn cant_move_king_within_check_ray() {
        let fen = "4K3/4R3/8/8/8/8/4k3/8 b - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let legal_moves = game.legal_moves();
        let looking_for = [
            Move::infer(Square::E2, Square::E1, &game),
            Move::infer(Square::E2, Square::E3, &game),
        ];

        for m in looking_for {
            shouldnt_generate(&legal_moves, &m);
        }
    }

    #[test]
    fn must_move_out_of_double_check() {
        let fen = "4k3/4r3/8/6Qb/8/2R5/4KP2/8 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let legal_moves = game.legal_moves();
        let king = Square::E2;

        for m in legal_moves {
            assert_eq!(
                m.from(game.turn),
                king,
                "Tried to move piece other than queen while double check. {}",
                m
            );
        }
    }

    fn ensure_legal_game(mut game: Game, game_turns: &[(Square, Square)]) {
        let mut move_num = 0;
        let mut psuedo_illegal_moves = HashMap::new();
        let mut illegal_moves = HashMap::new();
        for (i, to_play) in game_turns.iter().enumerate() {
            let to_play = Move::infer(to_play.0, to_play.1, &game);
            let to_play_from = to_play.from(game.turn);
            let fen = game.to_fen();
            let psuedo_legal_moves = game.generate_all_psuedo_legal_moves();
            let legal_moves = game.legal_moves();

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

            // let color = game.turn;
            let (piece, color) = if let Some(stuff) = game.piece_lookup(to_play_from) {
                stuff
            } else {
                let short = format!(
                    "Move: {}, Turn: {}. Tried to move nonexistant piece at square: {}\n  {}",
                    move_num, turn, to_play_from, fen
                );
                let long = short.clone();
                psuedo_illegal_moves.insert(short, long);
                break;
            };

            let piece_attacks = BitBoard::from_square_vec(moves_to_targets_vec(
                &piece.psuedo_legal_moves(&game, &to_play_from),
                &game,
            ));

            let piece_attacks_legal = BitBoard::from_square_vec(moves_to_targets_vec(
                &piece.legal_moves(&game, &to_play_from),
                &game,
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
ray_attacks:
{}

attacks:
{}

Black Board info:
ray_attacks:
{}

attacks:
{}

Available moves: {}
",
                    piece,
                    color,
                    to_play_from,
                    to_play.to(&game),
                    piece_attacks,
                    piece_attacks_legal,
                    game.white_check_rays,
                    game.white_attacks,
                    game.black_check_rays,
                    game.black_attacks,
                    format_pretty_list(&legal_moves)
                );

                illegal_moves.insert(short, long);
            }

            game.play(&to_play);
        }

        match psuedo_illegal_moves.len() {
            0 => {}
            1 => {
                let (short, long) = psuedo_illegal_moves.iter().next().unwrap();
                panic!("{}\n{}", short, long);
            }
            _ => {
                for short in psuedo_illegal_moves.keys() {
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
                for short in illegal_moves.keys() {
                    println!("{}", short);
                }
                panic!("{} illegal moves were found", illegal_moves.len());
            }
        }
    }

    /// https://www.chessgames.com/perl/chessgame?gid=1242968
    #[test]
    fn queens_gambit_game() {
        let game = Game::default();
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

        ensure_legal_game(game, &game_turns);
    }

    /// https://www.chessgames.com/perl/chessgame?gid=1955216
    #[test]
    fn sicilian_defense_game() {
        let game = Game::default();
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

        ensure_legal_game(game, &game_turns);
    }

    #[test]
    fn not_checkmate() {
        let fen = "r2q1rk1/p2n1pp1/1p3n1p/2b5/8/1R3P1N/P2pP1PP/2BQKB1R w K - 0 14";
        let mut game = Game::from_fen(fen).unwrap();
        let moves = game.legal_moves();
        let possible_moves = [
            Move::Normal {
                from: Square::D1,
                to: Square::D2,
                capture: Some(PieceType::Pawn),
            },
            Move::Normal {
                from: Square::C1,
                to: Square::D2,
                capture: Some(PieceType::Pawn),
            },
        ];

        assert_eq!(game.state, State::InProgress);
        should_generate(&moves, &possible_moves[0]);
        should_generate(&moves, &possible_moves[1]);
    }

    #[test]
    fn shouldnt_have_moves() {
        let fen = "1kb2b1r/1p1p1ppp/1Np5/8/4P1PP/1P3PK1/r6q/8 w - - 1 27";
        let mut game = Game::from_fen(fen).unwrap();
        let moves = game.legal_moves();
        assert!(
            moves.is_empty(),
            "White can play: {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn check_rays_are_populated() {
        let fen = "5B1n/8/1k2pR2/8/8/2K5/8/8 b - - 11 104";
        let game = Game::from_fen(fen).unwrap();
        println!("{:#?}", game);
        println!(
            "{}",
            format_pretty_list(&PieceType::Rook.psuedo_legal_moves(&game, &Square::F6))
        );
        println!(
            "{:#?}",
            PieceType::Rook.psuedo_legal_targets_fast(&game, &Square::F6)
        );
        assert_ne!(game.white_check_rays, EMPTY);
    }
}
