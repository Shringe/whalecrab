use crate::{
    bitboard::{BitBoard, EMPTY},
    game::Game,
    movegen::{
        moves::{Move, MoveType},
        pieces::{
            bishop::Bishop, king::King, knight::Knight, pawn::Pawn, queen::Queen, rook::Rook,
        },
    },
    rank::Rank,
    square::Square,
};

#[derive(Debug, PartialEq, Clone, Hash, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent(&self) -> Color {
        match &self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn final_rank(&self) -> Rank {
        match &self {
            Color::White => Rank::Eighth,
            Color::Black => Rank::First,
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

#[derive(Debug, PartialEq, Clone)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

macro_rules! dispatch_method_to_pieces {
    ($self:expr, $game:expr, $square:expr, $method:ident) => {
        match $self {
            PieceType::Pawn => Pawn($square).$method($game),
            PieceType::Knight => Knight($square).$method($game),
            PieceType::Bishop => Bishop($square).$method($game),
            PieceType::Rook => Rook($square).$method($game),
            PieceType::Queen => Queen($square).$method($game),
            PieceType::King => King($square).$method($game),
        }
    };
}

macro_rules! dispatch_piece_method {
    ($method:ident, $out:ty, mut) => {
        pub fn $method(&self, game: &mut Game, square: Square) -> $out {
            dispatch_method_to_pieces!(self, game, square, $method)
        }
    };
    ($method:ident, $out:ty) => {
        pub fn $method(&self, game: &Game, square: Square) -> $out {
            dispatch_method_to_pieces!(self, game, square, $method)
        }
    };
}

impl PieceType {
    dispatch_piece_method!(psuedo_legal_moves, Vec<Move>, mut);
    dispatch_piece_method!(legal_moves, Vec<Move>, mut);
    dispatch_piece_method!(psuedo_legal_targets_fast, PieceMoveInfo);

    pub fn is_ray_piece(&self) -> bool {
        match self {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => true,
            _ => false,
        }
    }
}

/// Stores where a piece could move to and what squares it currently defends
#[derive(Default)]
pub struct PieceMoveInfo {
    /// The possible squares a piece can move to
    pub targets: BitBoard,
    /// The squares a piece attacks/defends
    pub attacks: BitBoard,
    /// Pins for ray pieces. Empty if not a ray piece
    pub check_rays: BitBoard,
}

pub trait Piece {
    /// Generates psuedo legal moves not considering king safety.
    fn psuedo_legal_moves(&self, game: &Game) -> Vec<Move>;

    /// Generates the attack and target board for a piece without updating game
    /// Warning: Highly expiremental
    fn psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo;

    /// Generates legal moves considering king safety.
    fn legal_moves(&self, game: &Game) -> Vec<Move> {
        let psuedo_legal = self.psuedo_legal_moves(game);
        let mut legal = Vec::new();

        let enemy = game.position.turn.opponent();
        let attack_board = game.get_attacks(&enemy);
        let check_ray_board = game.get_check_rays(&enemy);

        let kingbb = game.get_pieces(&PieceType::King, &game.position.turn);
        let king = kingbb.to_square();
        let num_checks = game.num_attackers(king);

        for m in psuedo_legal {
            let frombb = BitBoard::from_square(m.from);
            let tobb = BitBoard::from_square(m.to);
            let (piece, _) = game
                .determine_piece(&frombb)
                .expect("Can't move nonexisting piece!");

            let is_moving_king = piece == PieceType::King;
            let is_capturing = matches!(m.variant, MoveType::Capture(_));
            let checks = game.get_check_rays(&enemy);
            let is_blocking = checks & tobb != EMPTY;

            // Handle being in check
            match num_checks {
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        board::{Board, State},
        movegen::moves::get_targets,
        square::Square,
        test_utils::{format_pretty_list, should_generate, shouldnt_generate},
    };

    use super::*;

    #[test]
    fn cant_move_into_check() {
        let fen = "1k6/1r6/8/8/8/8/8/K7 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let psuedo_legal = game.generate_all_psuedo_legal_moves();
        let legal = game.generate_all_legal_moves();

        let legal_looking_for = vec![Move::new(Square::A1, Square::A2, &game.position)];
        let psuedo_legal_looking_for = vec![
            Move::new(Square::A1, Square::A2, &game.position),
            Move::new(Square::A1, Square::B1, &game.position),
            Move::new(Square::A1, Square::B2, &game.position),
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
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());

        let legal_moves = game.generate_all_legal_moves();
        let looking_for = Move::new(Square::C4, Square::E3, &game.position);

        should_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn must_move_out_of_check() {
        let fen = "4k3/4r3/8/8/8/3P1P2/4KP2/3RRR2 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());

        let legal_moves = game.generate_all_legal_moves();
        let looking_for = [Move::new(Square::E2, Square::D2, &game.position)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn capture_checking_piece() {
        let fen = "4k3/4r3/8/8/1B6/3P1P2/3PKP2/3RRR2 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());

        let legal_moves = game.generate_all_legal_moves();
        let looking_for = [Move::new(Square::B4, Square::E7, &game.position)];

        assert_eq!(legal_moves, looking_for);
    }

    #[test]
    fn pinned_piece_cannot_move() {
        let fen = "4k3/4r3/8/8/3P1P2/4B3/3PK3/6P1 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());

        let legal_moves = game.generate_all_legal_moves();
        let looking_for = Move::new(Square::E3, Square::F2, &game.position);

        shouldnt_generate(&legal_moves, &looking_for);
    }

    #[test]
    fn cant_move_king_within_check_ray() {
        let fen = "4K3/4R3/8/8/8/8/4k3/8 b - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());

        let legal_moves = game.generate_all_legal_moves();
        let looking_for = [
            Move::new(Square::E2, Square::E1, &game.position),
            Move::new(Square::E2, Square::E3, &game.position),
        ];

        for m in looking_for {
            shouldnt_generate(&legal_moves, &m);
        }
    }

    #[test]
    fn must_move_out_of_double_check() {
        let fen = "4k3/4r3/8/6Qb/8/2R5/4KP2/8 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let legal_moves = game.generate_all_legal_moves();
        let king = Square::E2;

        for m in legal_moves {
            assert_eq!(
                m.from, king,
                "Tried to move piece other than queen while double check. {}",
                m
            );
        }
    }

    fn ensure_legal_game(mut game: Game, game_turns: &Vec<(Square, Square)>) {
        let mut move_num = 0;
        let mut psuedo_illegal_moves = HashMap::new();
        let mut illegal_moves = HashMap::new();
        for (i, to_play) in game_turns.iter().enumerate() {
            let to_play = Move::new(to_play.0, to_play.1, &game.position);
            let frombb = BitBoard::from_square(to_play.from);
            let fen = game.position.to_fen();
            let psuedo_legal_moves = game.generate_all_psuedo_legal_moves();
            let legal_moves = game.generate_all_legal_moves();

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

            // let color = game.position.turn;
            let (piece, color) = if let Some(stuff) = game.determine_piece(&frombb) {
                stuff
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
                piece.psuedo_legal_moves(&mut game, to_play.from),
            ));

            let piece_attacks_legal =
                BitBoard::from_square_vec(get_targets(piece.legal_moves(&mut game, to_play.from)));

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
                    game.white_num_checks,
                    game.white_check_rays,
                    game.white_attacks,
                    game.black_num_checks,
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
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let moves = game.generate_all_legal_moves();
        let possible_moves = vec![
            Move {
                from: Square::D1,
                to: Square::D2,
                variant: MoveType::Capture(PieceType::Pawn),
            },
            Move {
                from: Square::C1,
                to: Square::D2,
                variant: MoveType::Capture(PieceType::Pawn),
            },
        ];

        assert_eq!(game.position.state, State::InProgress);
        should_generate(&moves, &possible_moves[0]);
        should_generate(&moves, &possible_moves[1]);
    }
}
