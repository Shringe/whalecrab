use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{color_field_getters, Board, Color, PieceType, State},
    castling::{self, CastleSide},
    movegen::moves::{get_targets, Move, MoveType},
    square::Square,
};

#[derive(Clone)]
pub struct Game {
    pub position: Board,
    pub white_occupied: BitBoard,
    pub black_occupied: BitBoard,
    pub occupied: BitBoard,
    pub pawns: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub rooks: BitBoard,
    pub queens: BitBoard,
    pub kings: BitBoard,

    pub transposition_table: HashMap<u64, f32>,
    pub white_num_checks: u8,
    pub black_num_checks: u8,
    pub white_attacks: BitBoard,
    pub black_attacks: BitBoard,
    pub white_check_rays: BitBoard,
    pub black_check_rays: BitBoard,
}

impl Default for Game {
    fn default() -> Self {
        Self::from_position(Board::default())
    }
}

impl Game {
    color_field_getters!(attacks, BitBoard);
    color_field_getters!(check_rays, BitBoard);
    color_field_getters!(num_checks, u8);
    color_field_getters!(occupied, BitBoard);

    pub fn from_position(position: Board) -> Self {
        let mut game = Self {
            position,
            transposition_table: HashMap::new(),
            white_num_checks: 0,
            black_num_checks: 0,
            white_attacks: EMPTY,
            black_attacks: EMPTY,
            white_check_rays: EMPTY,
            black_check_rays: EMPTY,
            white_occupied: EMPTY,
            black_occupied: EMPTY,
            occupied: EMPTY,
            pawns: EMPTY,
            knights: EMPTY,
            bishops: EMPTY,
            rooks: EMPTY,
            queens: EMPTY,
            kings: EMPTY,
        };

        game.reinitialize();
        game
    }

    /// Recalculates certain cached values regarding the position
    /// Should be called on Self initialization and position updates
    fn refresh(&mut self) {
        let white_pieces = self.position.white_pawns
            | self.position.white_knights
            | self.position.white_bishops
            | self.position.white_rooks
            | self.position.white_queens
            | self.position.white_kings;
        let black_pieces = self.position.black_pawns
            | self.position.black_knights
            | self.position.black_bishops
            | self.position.black_rooks
            | self.position.black_queens
            | self.position.black_kings;
        let pieces = white_pieces | black_pieces;

        let pawns = self.position.white_pawns | self.position.black_pawns;
        let knights = self.position.white_knights | self.position.black_knights;
        let bishops = self.position.white_bishops | self.position.black_bishops;
        let rooks = self.position.white_rooks | self.position.black_rooks;
        let queens = self.position.white_queens | self.position.black_queens;
        let kings = self.position.white_kings | self.position.black_kings;

        self.pawns = pawns;
        self.knights = knights;
        self.bishops = bishops;
        self.rooks = rooks;
        self.queens = queens;
        self.kings = kings;
        self.white_occupied = white_pieces;
        self.black_occupied = black_pieces;
        self.occupied = pieces;

        let mut hasher = DefaultHasher::new();
        self.position.hash(&mut hasher);
        self.position.hash = hasher.finish();
        if let Some(times_seen) = self.position.seen_positions.get_mut(&self.position.hash) {
            *times_seen += 1;
        } else {
            self.position.seen_positions.insert(self.position.hash, 1);
        }
    }

    /// Reinitializes the game and its metadata. This is slow and unnecessary if you generate each
    /// move before playing it through self.generate(_psuedo)_legal_moves()
    pub fn reinitialize(&mut self) {
        self.refresh();

        // HACK: populating check and attacks boards
        self.generate_all_psuedo_legal_moves();
        self.position.turn = self.position.turn.opponent();
        self.generate_all_psuedo_legal_moves();
        self.position.turn = self.position.turn.opponent();
    }

    /// Finishes a turn and determines game state is possible
    fn next_turn(&mut self, last_move: &Move) {
        // Handle en_passant
        self.position.en_passant_target = if last_move.variant == MoveType::CreateEnPassant {
            last_move.to.backward(&self.position.turn)
        } else {
            None
        };

        // Update position state
        self.position.turn = self.position.turn.opponent();
        self.position.half_move_clock += 1;
        self.refresh();

        // Repetition
        if *self
            .position
            .seen_positions
            .get(&self.position.hash)
            .expect("Position should be hashed!")
            == 3
        {
            self.position.state = dbg!(State::Repetition);
        }

        // Half move timeout
        if matches!(last_move.variant, MoveType::Capture(_))
            || matches!(
                self.determine_piece(&BitBoard::from_square(last_move.to)),
                Some((PieceType::Pawn, _))
            )
        {
            self.position.half_move_timeout = 0;
        } else {
            self.position.half_move_timeout += 1;
        }

        if self.position.half_move_timeout == 100 {
            self.position.state = dbg!(State::Timeout);
        }
    }

    /// Gets the bitboard of a colored piece
    pub fn get_pieces_mut(&mut self, piece: &PieceType, color: &Color) -> &mut BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => &mut self.position.white_pawns,
                PieceType::Knight => &mut self.position.white_knights,
                PieceType::Bishop => &mut self.position.white_bishops,
                PieceType::Rook => &mut self.position.white_rooks,
                PieceType::Queen => &mut self.position.white_queens,
                PieceType::King => &mut self.position.white_kings,
            },
            Color::Black => match piece {
                PieceType::Pawn => &mut self.position.black_pawns,
                PieceType::Knight => &mut self.position.black_knights,
                PieceType::Bishop => &mut self.position.black_bishops,
                PieceType::Rook => &mut self.position.black_rooks,
                PieceType::Queen => &mut self.position.black_queens,
                PieceType::King => &mut self.position.black_kings,
            },
        }
    }

    /// Determines color of standing piece
    pub fn determine_color(&self, sqbb: &BitBoard) -> Option<Color> {
        if self.white_occupied.has_square(sqbb) {
            Some(Color::White)
        } else if self.black_occupied.has_square(sqbb) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Determines type and color of standing piece
    pub fn determine_piece(&self, sqbb: &BitBoard) -> Option<(PieceType, Color)> {
        if self.white_occupied.has_square(sqbb) {
            if self.position.white_pawns.has_square(sqbb) {
                Some((PieceType::Pawn, Color::White))
            } else if self.position.white_knights.has_square(sqbb) {
                Some((PieceType::Knight, Color::White))
            } else if self.position.white_bishops.has_square(sqbb) {
                Some((PieceType::Bishop, Color::White))
            } else if self.position.white_rooks.has_square(sqbb) {
                Some((PieceType::Rook, Color::White))
            } else if self.position.white_queens.has_square(sqbb) {
                Some((PieceType::Queen, Color::White))
            } else if self.position.white_kings.has_square(sqbb) {
                Some((PieceType::King, Color::White))
            } else {
                unreachable!("The white occupied bitboard has a square that no white pieces have!")
            }
        } else if self.black_occupied.has_square(sqbb) {
            if self.position.black_pawns.has_square(sqbb) {
                Some((PieceType::Pawn, Color::Black))
            } else if self.position.black_knights.has_square(sqbb) {
                Some((PieceType::Knight, Color::Black))
            } else if self.position.black_bishops.has_square(sqbb) {
                Some((PieceType::Bishop, Color::Black))
            } else if self.position.black_rooks.has_square(sqbb) {
                Some((PieceType::Rook, Color::Black))
            } else if self.position.black_queens.has_square(sqbb) {
                Some((PieceType::Queen, Color::Black))
            } else if self.position.black_kings.has_square(sqbb) {
                Some((PieceType::King, Color::Black))
            } else {
                unreachable!("The black occupied bitboard has a square that no black pieces have!")
            }
        } else {
            None
        }
    }

    /// Plays a move on the board, updating the position and engine values
    pub fn play(&mut self, m: &Move) {
        let frombb = BitBoard::from_square(m.from);
        let tobb = BitBoard::from_square(m.to);
        let (piece, color) = self
            .determine_piece(&frombb)
            .expect("Couldn't find piece to move!");
        let (enemy_piece, enemy_color) = match self.determine_piece(&tobb) {
            Some((piece, color)) => (Some(piece), color),
            None => {
                let enemy_piece = if m.variant == MoveType::CaptureEnPassant {
                    Some(PieceType::Pawn)
                } else {
                    None
                };

                (enemy_piece, color.opponent())
            }
        };

        if let MoveType::Castle(side) = &m.variant {
            match &color {
                Color::White => {
                    self.position.castling_rights.white_queenside = false;
                    self.position.castling_rights.white_kingside = false;

                    match side {
                        CastleSide::Queenside => {
                            self.position.white_kings ^=
                                castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                            self.position.white_rooks ^=
                                castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            self.position.white_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                            self.position.white_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    }
                }

                Color::Black => {
                    self.position.castling_rights.black_queenside = false;
                    self.position.castling_rights.black_kingside = false;

                    match side {
                        CastleSide::Queenside => {
                            self.position.black_kings ^=
                                castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                            self.position.black_rooks ^=
                                castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            self.position.black_kings ^= castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                            self.position.black_rooks ^= castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    }
                }
            }

            self.next_turn(&m);
            return;
        }

        // Update attack bitboards
        match piece {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
                // HACK: Clone so that attack boards are not automatically updated for now
                // TODO: Implement way to movegen withhout setting attack boards
                let attack_board = *self.get_attacks(&color);
                let check_ray_board = *self.get_check_rays(&color);
                let moves = piece.get_psuedo_legal_moves(self, m.from);
                let initial_check_ray = BitBoard::from_square_vec(get_targets(moves));

                *self.get_attacks_mut(&color) = attack_board ^ initial_check_ray;
                *self.get_check_rays_mut(&color) = check_ray_board;
            }
            PieceType::King => {
                *self.get_check_rays_mut(&enemy_color) = EMPTY;
            }
            _ => {}
        }

        // Remove the piece from its original square
        let pieces = self.get_pieces_mut(&piece, &color);
        *pieces ^= frombb;

        // Add the piece to the new square
        if let MoveType::Promotion(piece) = &m.variant {
            let pieces = self.get_pieces_mut(piece, &color);
            *pieces |= tobb;
        } else {
            *pieces |= tobb;
        }

        // Capture if available
        if let Some(piece) = &enemy_piece {
            let pieces = self.get_pieces_mut(piece, &enemy_color);
            if m.variant == MoveType::CaptureEnPassant {
                let en_passant_bb = BitBoard::from_square(
                    m.to.backward(&color)
                        .expect("Can't find pawn in front of en_passant_target!"),
                );
                *pieces ^= en_passant_bb;
            } else {
                *pieces ^= tobb;
            }
        }

        // Revoke castling rights if something moves on a critical square
        let mut revoke_rights = |square: &Square| match square {
            &Square::E1 => {
                self.position.castling_rights.white_kingside = false;
                self.position.castling_rights.white_queenside = false;
            }
            &Square::A1 => self.position.castling_rights.white_queenside = false,
            &Square::H1 => self.position.castling_rights.white_kingside = false,
            &Square::E8 => {
                self.position.castling_rights.black_kingside = false;
                self.position.castling_rights.black_queenside = false;
            }
            &Square::A8 => self.position.castling_rights.black_queenside = false,
            &Square::H8 => self.position.castling_rights.black_kingside = false,
            _ => {}
        };

        revoke_rights(&m.from);
        revoke_rights(&m.to);

        self.next_turn(&m);
    }

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = self.get_occupied(&self.position.turn);

        for sq in *occupied {
            let sqbb = BitBoard::from_square(sq);
            if let Some((piece, _)) = self.determine_piece(&sqbb) {
                moves.extend(piece.get_psuedo_legal_moves(self, sq))
            }
        }

        moves
    }

    /// Generates all legal moves for the current player. This also updates position state
    /// for statemate or checkmate
    pub fn generate_all_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        if self.position.state != State::InProgress {
            return moves;
        }

        let occupied = self.get_occupied(&self.position.turn);

        for sq in *occupied {
            let sqbb = BitBoard::from_square(sq);
            if let Some((piece, _)) = self.determine_piece(&sqbb) {
                moves.extend(piece.get_legal_moves(self, sq))
            }
        }

        if moves.is_empty() {
            self.position.state = if *self.get_num_checks(&self.position.turn) > 0 {
                dbg!(State::Checkmate)
            } else {
                dbg!(State::Stalemate)
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Color, PieceType, State};
    use crate::castling::{BLACK_CASTLES_KINGSIDE, WHITE_CASTLES_QUEENSIDE};
    use crate::game::Game;
    use crate::movegen::moves::{Move, MoveType};
    use crate::movegen::pieces::king::King;
    use crate::movegen::pieces::pawn::Pawn;
    use crate::movegen::pieces::piece::Piece;
    use crate::square::Square;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

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
        let original_position = game.position.clone();

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

        // Test pawn move
        game.play(&pawn);
        let after_pawn = game.position.clone();

        // Reset and test knight move
        game.position = original_position.clone();
        game.play(&knight);
        let after_knight = game.position.clone();

        // Reset and test king move
        game.position = original_position.clone();
        game.play(&king);
        let after_king = game.position.clone();

        assert!(pawn.from.in_bitboard(&original_position.white_pawns));
        assert!(!pawn.to.in_bitboard(&original_position.white_pawns));
        assert!(!pawn.from.in_bitboard(&after_pawn.white_pawns));
        assert!(pawn.to.in_bitboard(&after_pawn.white_pawns));

        assert!(knight.from.in_bitboard(&original_position.black_knights));
        assert!(!knight.to.in_bitboard(&original_position.black_knights));
        assert!(!knight.from.in_bitboard(&after_knight.black_knights));
        assert!(knight.to.in_bitboard(&after_knight.black_knights));

        assert!(king.from.in_bitboard(&original_position.white_kings));
        assert!(!king.to.in_bitboard(&original_position.white_kings));
        assert!(!king.from.in_bitboard(&after_king.white_kings));
        assert!(king.to.in_bitboard(&after_king.white_kings));
    }

    #[test]
    fn white_pawn_captures_black_pawn() {
        let mut game = Game::default();
        let capture = Move {
            from: Square::B4,
            to: Square::C5,
            variant: MoveType::Normal,
        };
        let white_pawns_before = game.position.white_pawns.popcnt();
        let black_pawns_before = game.position.black_pawns.popcnt();

        for m in [
            Move {
                from: Square::B2,
                to: Square::B4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::C7,
                to: Square::C5,
                variant: MoveType::Normal,
            },
            capture.clone(),
        ] {
            game.play(&m);
        }

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

    #[test]
    fn white_gets_checkmated() {
        let fen = "2r5/8/8/8/8/8/5k2/7K w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        assert_eq!(game.position.state, State::InProgress);
        let moves = [(Square::H1, Square::H2), (Square::C8, Square::H8)];
        for (from, to) in moves {
            let m = Move::new(from, to, &game.position);
            game.play(&m);
        }

        game.reinitialize();
        game.generate_all_legal_moves();
        assert_eq!(game.position.turn, Color::White);
        assert_eq!(game.position.state, State::Checkmate);
    }

    #[test]
    fn black_gets_stalmated() {
        let fen = "4k3/4P3/5K2/8/8/8/8/8 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let to_play = Move::new(Square::F6, Square::E6, &game.position);

        assert_eq!(game.position.state, State::InProgress);
        should_generate(&game.generate_all_legal_moves(), &to_play);
        game.play(&to_play);
        game.reinitialize();
        game.generate_all_legal_moves();
        assert_eq!(game.position.turn, Color::Black);
        assert_eq!(game.position.state, State::Stalemate);
    }

    #[test]
    fn draw_fifty_move_rule() {
        todo!("Fen parsing needs to handle move count")
    }

    #[test]
    fn draw_by_repitition() {
        let mut game = Game::default();
        let moves = [
            (Square::G1, Square::F3),
            (Square::B8, Square::C6),
            (Square::F3, Square::G1),
            (Square::C6, Square::B8),
            (Square::G1, Square::F3),
            (Square::B8, Square::C6),
            (Square::F3, Square::G1),
            (Square::C6, Square::B8),
        ];

        for (from, to) in moves {
            assert_eq!(game.position.state, State::InProgress);
            let m = Move::new(from, to, &game.position);
            should_generate(&game.generate_all_legal_moves(), &m);
            game.play(&m);
        }

        game.generate_all_legal_moves();
        assert_eq!(game.position.state, State::Repetition);
    }
}
