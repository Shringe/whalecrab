use std::{fmt, str::FromStr};

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::Board,
    castling::{self, CastleSide},
    game::Game,
    movegen::pieces::piece::{Color, PieceType},
    square::Square,
};

#[derive(PartialEq, Debug, Clone)]
pub enum MoveType {
    Normal,
    Capture(PieceType),
    CreateEnPassant,
    CaptureEnPassant,
    Promotion(PieceType),
    Castle(CastleSide),
}

/// Converts a vector of moves to a vector of taargets
pub fn get_targets(moves: Vec<Move>) -> Vec<Square> {
    moves.into_iter().map(|m| m.to).collect()
}

#[derive(PartialEq, Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub variant: MoveType,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}, {:?}", self.from, self.to, self.variant)
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Move {
    /// Infers the type of variant move. This is likely already known during move generation, and in that
    /// case it is recommended to skip using this contructor.
    pub fn new(from: Square, to: Square, board: &Board) -> Self {
        Self {
            from,
            to,
            variant: match (&board.turn, from, to) {
                (Color::White, Square::E1, Square::C1) if board.castling_rights.white_queenside => {
                    MoveType::Castle(CastleSide::Queenside)
                }
                (Color::White, Square::E1, Square::G1) if board.castling_rights.white_kingside => {
                    MoveType::Castle(CastleSide::Kingside)
                }
                (Color::Black, Square::E8, Square::C8) if board.castling_rights.black_queenside => {
                    MoveType::Castle(CastleSide::Queenside)
                }
                (Color::Black, Square::E8, Square::G8) if board.castling_rights.black_kingside => {
                    MoveType::Castle(CastleSide::Kingside)
                }
                _ => {
                    if let Some(enemy) = board.determine_piece(to) {
                        MoveType::Capture(enemy)
                    } else if board.determine_piece(from) == Some(PieceType::Pawn) {
                        let color = board.determine_color(from).unwrap();
                        if Some(to) == board.en_passant_target {
                            MoveType::CaptureEnPassant
                        } else if let Some(once) = from.forward(&color) {
                            if let Some(twice) = once.forward(&color) {
                                if to == twice {
                                    MoveType::CreateEnPassant
                                } else {
                                    MoveType::Normal
                                }
                            } else if once.get_rank() == color.final_rank() {
                                MoveType::Promotion(PieceType::Queen)
                            } else if let Some(enemy) = board.determine_piece(to) {
                                MoveType::Capture(enemy)
                            } else {
                                MoveType::Normal
                            }
                        } else {
                            MoveType::Normal
                        }
                    } else {
                        MoveType::Normal
                    }
                }
            },
        }
    }

    /// Formats the move in uci notation, such as e2e4
    pub fn to_uci(&self) -> String {
        format!(
            "{}{}",
            self.from.to_string().to_lowercase(),
            self.to.to_string().to_lowercase()
        )
    }

    /// Returns a move from a uci string
    pub fn from_uci(uci: &str, position: &Board) -> Result<Self, ()> {
        Ok(Move::new(
            Square::from_str(&uci[..2])?,
            Square::from_str(&uci[2..])?,
            position,
        ))
    }

    fn play_normal(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");

        self.update_attack_boards(game, &piece, &color);
        self.move_piece(game, &piece, &color, frombb, tobb);
        self.revoke_castling_rights(game);
        game.next_turn(&self);
    }

    fn play_capture(&self, game: &mut Game, piece_type: &PieceType) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");
        let enemy_color = color.opponent();

        self.update_attack_boards(game, &piece, &color);
        self.move_piece(game, &piece, &color, frombb, tobb);

        // Capture the enemy piece
        let enemy_pieces = game.get_pieces_mut(piece_type, &enemy_color);
        *enemy_pieces ^= tobb;

        self.revoke_castling_rights(game);
        game.next_turn(&self);
    }

    fn play_create_en_passant(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");

        self.update_attack_boards(game, &piece, &color);
        self.move_piece(game, &piece, &color, frombb, tobb);
        self.revoke_castling_rights(game);
        game.next_turn(&self);
    }

    fn play_capture_en_passant(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");
        let enemy_color = color.opponent();

        self.update_attack_boards(game, &piece, &color);
        self.move_piece(game, &piece, &color, frombb, tobb);

        // Capture the pawn en passant
        let en_passant_bb = BitBoard::from_square(
            self.to
                .backward(&color)
                .expect("Can't find pawn in front of en_passant_target!"),
        );
        let enemy_pawns = game.get_pieces_mut(&PieceType::Pawn, &enemy_color);
        *enemy_pawns ^= en_passant_bb;

        self.revoke_castling_rights(game);
        game.next_turn(&self);
    }

    fn play_promotion(&self, game: &mut Game, promoted_piece: &PieceType) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");

        self.update_attack_boards(game, &piece, &color);

        // Remove pawn from original square
        let pieces = game.get_pieces_mut(&piece, &color);
        *pieces ^= frombb;

        // Add promoted piece to new square
        let promoted_pieces = game.get_pieces_mut(promoted_piece, &color);
        *promoted_pieces |= tobb;

        self.revoke_castling_rights(game);
        game.next_turn(&self);
    }

    fn play_castle(&self, game: &mut Game, castle_side: &CastleSide) {
        let frombb = BitBoard::from_square(self.from);
        let (_, color) = game
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");

        match &color {
            Color::White => {
                game.position.castling_rights.white_queenside = false;
                game.position.castling_rights.white_kingside = false;

                match castle_side {
                    CastleSide::Queenside => {
                        game.position.white_kings ^= castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                        game.position.white_rooks ^= castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                    }
                    CastleSide::Kingside => {
                        game.position.white_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                        game.position.white_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                    }
                }
            }

            Color::Black => {
                game.position.castling_rights.black_queenside = false;
                game.position.castling_rights.black_kingside = false;

                match castle_side {
                    CastleSide::Queenside => {
                        game.position.black_kings ^= castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                        game.position.black_rooks ^= castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                    }
                    CastleSide::Kingside => {
                        game.position.black_kings ^= castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                        game.position.black_rooks ^= castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                    }
                }
            }
        }

        game.next_turn(&self);
    }

    /// Helper method to update attack boards
    fn update_attack_boards(&self, game: &mut Game, piece: &PieceType, color: &Color) {
        let enemy_color = color.opponent();

        match piece {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
                // HACK
                let attack_board = *game.get_attacks(&color);
                let check_ray_board = *game.get_check_rays(&color);
                let moves = piece.psuedo_legal_moves(game, self.from);
                let initial_check_ray = BitBoard::from_square_vec(get_targets(moves));

                *game.get_attacks_mut(&color) = attack_board ^ initial_check_ray;
                *game.get_check_rays_mut(&color) = check_ray_board;
            }
            PieceType::King => {
                *game.get_check_rays_mut(&enemy_color) = EMPTY;
            }
            _ => {}
        }
    }

    /// Helper method to move a piece from one square to another
    fn move_piece(
        &self,
        game: &mut Game,
        piece: &PieceType,
        color: &Color,
        frombb: BitBoard,
        tobb: BitBoard,
    ) {
        let pieces = game.get_pieces_mut(piece, color);
        *pieces ^= frombb;
        *pieces |= tobb;
    }

    /// Helper method to revoke castling rights based on move squares
    fn revoke_castling_rights(&self, game: &mut Game) {
        let mut revoke_rights = |square: &Square| match square {
            &Square::E1 => {
                game.position.castling_rights.white_kingside = false;
                game.position.castling_rights.white_queenside = false;
            }
            &Square::A1 => game.position.castling_rights.white_queenside = false,
            &Square::H1 => game.position.castling_rights.white_kingside = false,
            &Square::E8 => {
                game.position.castling_rights.black_kingside = false;
                game.position.castling_rights.black_queenside = false;
            }
            &Square::A8 => game.position.castling_rights.black_queenside = false,
            &Square::H8 => game.position.castling_rights.black_kingside = false,
            _ => {}
        };

        revoke_rights(&self.from);
        revoke_rights(&self.to);
    }

    /// Plays a move on the board
    pub fn play(&self, game: &mut Game) {
        match &self.variant {
            MoveType::Normal => self.play_normal(game),
            MoveType::Capture(piece_type) => self.play_capture(game, piece_type),
            MoveType::CreateEnPassant => self.play_create_en_passant(game),
            MoveType::CaptureEnPassant => self.play_capture_en_passant(game),
            MoveType::Promotion(piece_type) => self.play_promotion(game, piece_type),
            MoveType::Castle(castle_side) => self.play_castle(game, castle_side),
        }
    }
}

impl Game {
    /// Plays a move on the board, updating the position and engine values
    pub fn play(&mut self, m: &Move) {
        m.play(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::castling::{BLACK_CASTLES_KINGSIDE, WHITE_CASTLES_QUEENSIDE};
    use crate::game::Game;
    use crate::movegen::pieces::king::King;
    use crate::movegen::pieces::pawn::Pawn;
    use crate::movegen::pieces::piece::Piece;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

    #[test]
    fn to_uci() {
        let uci = "e2e4";
        let m = Move {
            from: Square::E2,
            to: Square::E4,
            variant: MoveType::Normal,
        };

        assert_eq!(m.to_uci(), uci.to_owned());
    }

    #[test]
    fn from_uci() {
        let game = Game::default();
        let uci = "e2e4";
        let m = Move {
            from: Square::E2,
            to: Square::E4,
            variant: MoveType::CreateEnPassant,
        };

        assert_eq!(Move::from_uci(uci, &game.position).unwrap(), m);
    }

    #[test]
    fn both_lose_castling_rights_by_moving_kings() {
        let mut game = Game::from_position(
            Board::from_fen("rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1")
                .unwrap(),
        );

        let black_moves = Move::new(Square::E8, Square::D7, &game.position);
        game.play(&black_moves);

        compare_to_fen(
            &game.position,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQ - 1 2",
        );

        let white_moves = Move::new(Square::E1, Square::E2, &game.position);
        game.play(&white_moves);

        compare_to_fen(
            &game.position,
            "rnbq1b1r/pppkpppp/3p4/3nP3/3P4/5N2/PPP1KPPP/RNBQ1B1R b - - 2 2",
        );
    }

    #[test]
    fn both_lose_castling_rights_by_moving_rooks() {
        let mut game = Game::from_position(
            Board::from_fen("rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1")
                .unwrap(),
        );

        let black_moves = Move::new(Square::H8, Square::G8, &game.position);
        game.play(&black_moves);

        compare_to_fen(
            &game.position,
            "rnbqkbr1/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKB1R w KQq - 1 2",
        );

        let white_moves = Move::new(Square::H1, Square::G1, &game.position);
        game.play(&white_moves);

        compare_to_fen(
            &game.position,
            "rnbqkbr1/ppp1pppp/3p4/3nP3/3P4/5N2/PPP2PPP/RNBQKBR1 b Qq - 2 2",
        );
    }

    #[test]
    fn white_king_castles_queenside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/R3K1NR w KQkq - 6 7";
        let fen_after = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let to_play = &WHITE_CASTLES_QUEENSIDE;
        let mut game = Game::from_position(Board::from_fen(fen_before).unwrap());

        let moves = King(to_play.from).psuedo_legal_moves(&mut game);
        should_generate(&moves, to_play);

        game.play(to_play);
        compare_to_fen(&game.position, fen_after);
    }

    #[test]
    fn black_king_castles_kingside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let fen_after = "rn3rk1/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR w - - 8 8";
        let to_play = &BLACK_CASTLES_KINGSIDE;
        let mut game = Game::from_position(Board::from_fen(fen_before).unwrap());

        let moves = King(to_play.from).psuedo_legal_moves(&mut game);
        should_generate(&moves, to_play);

        game.play(to_play);
        compare_to_fen(&game.position, fen_after);
    }

    #[test]
    fn white_pawn_promotes_to_queen() {
        let mut game = Game::default();
        let looking_for = Move {
            from: Square::G7,
            to: Square::H8,
            variant: MoveType::Promotion(PieceType::Queen),
        };

        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::H4,
                to: Square::G5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H7,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::G5,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::F8,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H6,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::E7,
                to: Square::E5,
                variant: MoveType::CreateEnPassant,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.position.turn, Color::White);
        assert!(
            looking_for.from.in_bitboard(&game.position.white_pawns),
            "White pawn not in position"
        );
        assert!(
            looking_for.to.in_bitboard(&game.position.black_rooks),
            "Black rook not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );

        game.play(&looking_for);

        assert_eq!(game.position.turn, Color::Black);
        assert!(
            Square::H8.in_bitboard(&game.position.white_queens),
            "Expected white queen at H8 after promotion"
        );
        assert!(
            !Square::H8.in_bitboard(&game.position.white_pawns),
            "H8 incorrectly contains a white pawn after promotion"
        );
        assert!(
            !looking_for.from.in_bitboard(&game.position.white_pawns),
            "Original white pawn still present at {} after promotion",
            looking_for.from
        );
    }

    #[test]
    fn make_moves() {
        let mut game = Game::default();
        let original = game.position.clone();

        let pawn = Move {
            from: Square::C2,
            to: Square::C3,
            variant: MoveType::Normal,
        };
        let knight = Move {
            from: Square::G8,
            to: Square::F6,
            variant: MoveType::Normal,
        };
        let king = Move {
            from: Square::E1,
            to: Square::E2,
            variant: MoveType::Normal,
        };

        game.play(&pawn);
        let after_pawn = game.position.clone();

        game = Game::default();
        game.play(&knight);
        let after_knight = game.position.clone();

        game = Game::default();
        game.play(&king);
        let after_king = game.position.clone();

        assert!(pawn.from.in_bitboard(&original.white_pawns));
        assert!(!pawn.to.in_bitboard(&original.white_pawns));
        assert!(!pawn.from.in_bitboard(&after_pawn.white_pawns));
        assert!(pawn.to.in_bitboard(&after_pawn.white_pawns));

        assert!(knight.from.in_bitboard(&original.black_knights));
        assert!(!knight.to.in_bitboard(&original.black_knights));
        assert!(!knight.from.in_bitboard(&after_knight.black_knights));
        assert!(knight.to.in_bitboard(&after_knight.black_knights));

        assert!(king.from.in_bitboard(&original.white_kings));
        assert!(!king.to.in_bitboard(&original.white_kings));
        assert!(!king.from.in_bitboard(&after_king.white_kings));
        assert!(king.to.in_bitboard(&after_king.white_kings));
    }

    #[test]
    fn white_pawn_captures_black_pawn() {
        let mut game = Game::default();
        let white_pawns_before = game.position.white_pawns.popcnt();
        let black_pawns_before = game.position.black_pawns.popcnt();

        for (from, to) in [(Square::B2, Square::B4), (Square::C7, Square::C5)] {
            let m = Move::new(from, to, &game.position);
            game.play(&m);
        }

        let capture = Move::new(Square::B4, Square::C5, &game.position);
        game.play(&capture);

        let white_pawns_after = game.position.white_pawns.popcnt();
        let black_pawns_after = game.position.black_pawns.popcnt();

        assert!(
            !Square::B2.in_bitboard(&game.position.white_pawns),
            "White never moved"
        );
        assert!(
            !capture.from.in_bitboard(&game.position.white_pawns),
            "White moved but failed to capture"
        );
        assert!(
            !capture.to.in_bitboard(&game.position.black_pawns),
            "The black pawn is still standing"
        );
        assert!(
            capture.to.in_bitboard(&game.position.white_pawns),
            "White isn't in the correct position"
        );

        assert_ne!(
            black_pawns_before, black_pawns_after,
            "The number of black pawns didn't change"
        );
        assert_eq!(
            black_pawns_before - 1,
            black_pawns_after,
            "The number of black pawns didn't decrease by one"
        );
        assert_eq!(
            white_pawns_before, white_pawns_after,
            "The number of white pawns changed"
        );
    }

    #[test]
    fn black_pawn_takes_en_passant_target() {
        let mut game = Game::default();
        let capture = Move {
            from: Square::B4,
            to: Square::C3,
            variant: MoveType::CaptureEnPassant,
        };
        for m in [
            Move {
                from: Square::D2,
                to: Square::D3,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::B7,
                to: Square::B5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::D3,
                to: Square::D4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::B5,
                to: Square::B4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::C2,
                to: Square::C4,
                variant: MoveType::CreateEnPassant,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.position.turn, Color::Black);
        assert!(
            capture.from.in_bitboard(&game.position.black_pawns),
            "Black pawn not in position"
        );

        let moves = Pawn(capture.from).psuedo_legal_moves(&mut game);
        assert!(
            moves.contains(&capture),
            "Black pawn doesn't see en passant target. {}",
            format_pretty_list(&moves)
        );

        let white_pawns_before = game.position.white_pawns.popcnt();
        let black_pawns_before = game.position.black_pawns.popcnt();
        game.play(&capture);
        let white_pawns_after = game.position.white_pawns.popcnt();
        let black_pawns_after = game.position.black_pawns.popcnt();

        assert_eq!(black_pawns_before, black_pawns_after);
        assert_eq!(
            white_pawns_before - 1,
            white_pawns_after,
            "The white target is still standing"
        );
    }
}
