use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{color_field_getters, Board, State},
    castling::CastlingRights,
    movegen::{
        moves::{Move, MoveType},
        pieces::piece::{Color, PieceMoveInfo, PieceType, ALL_PIECE_TYPES},
    },
    square::Square,
};

/// Non-restoreable information needed to undo a move
#[derive(Clone, PartialEq, Debug)]
pub struct UnRestoreable {
    pub castling_rights: CastlingRights,
    pub half_move_timeout: usize,
}

#[derive(Clone, PartialEq)]
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
    pub position_history: Vec<UnRestoreable>,
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
            position_history: Vec::new(),
        };

        game.reinitialize();
        game
    }

    /// Restores the essential data from the previous position
    pub fn restore_position(&mut self) {
        let last_position = self
            .position_history
            .pop()
            .expect("Tried to unmake a move, but the required information is not present");
        self.position.castling_rights = last_position.castling_rights;
        self.position.half_move_timeout = last_position.half_move_timeout;
    }

    /// Captures essential position information to be restored later
    pub fn capture_position(&mut self) {
        let last_position = UnRestoreable {
            castling_rights: self.position.castling_rights.clone(),
            half_move_timeout: self.position.half_move_timeout.clone(),
        };
        self.position_history.push(last_position);
    }

    /// Recalculates certain cached values regarding the position
    /// Should be called on Self initialization and position updates
    pub fn refresh(&mut self) {
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

        self.update_attacks();
    }

    /// Calculates the attack bitboard for the given player
    fn calculate_attacks(&self, color: &Color) -> (BitBoard, BitBoard) {
        let mut attacks = EMPTY;
        let mut check_rays = EMPTY;

        for sq in *self.get_occupied(color) {
            let sqbb = BitBoard::from_square(sq);
            let (piece, _) = self.determine_piece(&sqbb).unwrap();
            let moveinfo = piece.psuedo_legal_targets_fast(&self, sq);
            attacks |= moveinfo.attacks;
            check_rays |= moveinfo.check_rays;
        }

        (attacks, check_rays)
    }

    /// Updates attack bitboard for the both players
    fn update_attacks(&mut self) {
        let color = self.position.turn;
        let enemy = color.opponent();

        (
            *self.get_attacks_mut(&color),
            *self.get_check_rays_mut(&color),
        ) = self.calculate_attacks(&color);

        (
            *self.get_attacks_mut(&enemy),
            *self.get_check_rays_mut(&enemy),
        ) = self.calculate_attacks(&enemy);
    }

    /// Reinitializes the game and its metadata. This is slow and unnecessary if you generate each
    /// move before playing it through self.generate(_psuedo)_legal_moves()
    pub fn reinitialize(&mut self) {
        self.refresh();
        // let enemy = self.position.turn;
        // *self.get_attacks_mut(&enemy) = self.calculate_attacks(&enemy);

        // HACK: populating check and attacks boards
        // self.generate_all_psuedo_legal_moves();
        // self.position.turn = self.position.turn.opponent();
        // self.generate_all_psuedo_legal_moves();
        // self.position.turn = self.position.turn.opponent();
    }

    /// Determines how many pieces are attacking a piece
    pub fn num_attackers(&self, sq: Square) -> u8 {
        let mut attackers = 0;
        let sqbb = BitBoard::from_square(sq);
        let color = if let Some(color) = self.determine_color(&sqbb) {
            color
        } else {
            return attackers;
        };

        let enemy = color.opponent();
        if !self.get_attacks(&enemy).has_square(&sqbb) {
            return attackers;
        }

        for piece in ALL_PIECE_TYPES {
            let moveinfo = piece.psuedo_legal_targets_fast(self, sq);
            let potential_enemy = self.get_pieces(&piece, &enemy);
            attackers += (moveinfo.attacks & potential_enemy).popcnt() as u8;
        }

        attackers
    }

    /// Finishes a turn and determines game state is possible
    pub fn next_turn(&mut self, last_move: &Move) {
        // Handle en_passant
        self.position.en_passant_target = if last_move.variant == MoveType::CreateEnPassant {
            last_move.to.backward(&self.position.turn)
        } else {
            None
        };

        // Update position state
        self.position.turn = self.position.turn.opponent();
        self.position.half_move_clock += 1;
        if self.position.turn == Color::Black {
            self.position.full_move_clock += 1;
        }
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

        if self.position.half_move_timeout == 50 {
            self.position.state = dbg!(State::Timeout);
        }
    }

    /// Reverses turn color and full_move_clock to the last turn
    pub fn previous_turn(&mut self, last_move: &Move) {
        self.position.en_passant_target = if last_move.variant == MoveType::CaptureEnPassant {
            Some(last_move.to)
        } else {
            None
        };

        self.position.turn = self.position.turn.opponent();
        if self.position.turn == Color::White {
            self.position.full_move_clock -= 1;
        }

        self.refresh();
    }

    /// Gets the bitboard of a colored piece
    pub fn get_pieces(&self, piece: &PieceType, color: &Color) -> &BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => &self.position.white_pawns,
                PieceType::Knight => &self.position.white_knights,
                PieceType::Bishop => &self.position.white_bishops,
                PieceType::Rook => &self.position.white_rooks,
                PieceType::Queen => &self.position.white_queens,
                PieceType::King => &self.position.white_kings,
            },
            Color::Black => match piece {
                PieceType::Pawn => &self.position.black_pawns,
                PieceType::Knight => &self.position.black_knights,
                PieceType::Bishop => &self.position.black_bishops,
                PieceType::Rook => &self.position.black_rooks,
                PieceType::Queen => &self.position.black_queens,
                PieceType::King => &self.position.black_kings,
            },
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

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_fast(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let color = &self.position.turn;
        let occupied = self.get_occupied(color);

        for sq in *occupied {
            let sqbb = BitBoard::from_square(sq);
            if let Some((piece, _)) = self.determine_piece(&sqbb) {
                let moveinfo = piece.psuedo_legal_targets_fast(&self, sq);
                for t in moveinfo.targets {
                    moves.push(Move::new(sq, t, &self.position));
                }
            }
        }

        moves
    }

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = self.get_occupied(&self.position.turn);

        for sq in *occupied {
            let sqbb = BitBoard::from_square(sq);
            if let Some((piece, _)) = self.determine_piece(&sqbb) {
                moves.extend(piece.psuedo_legal_moves(self, sq))
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
                moves.extend(piece.legal_moves(self, sq))
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
    use crate::bitboard::BitBoard;
    use crate::board::{Board, State};
    use crate::castling::{BLACK_CASTLES_KINGSIDE, WHITE_CASTLES_QUEENSIDE};
    use crate::game::Game;
    use crate::movegen::moves::{Move, MoveType};
    use crate::movegen::pieces::king::King;
    use crate::movegen::pieces::pawn::Pawn;
    use crate::movegen::pieces::piece::{Color, Piece, PieceType};
    use crate::square::Square;
    use crate::test_utils::{compare_to_fen, should_generate};

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
        let fen = "4k3/8/8/8/8/8/1NNN1KN1/8 w - - 49 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        assert_eq!(game.position.state, State::InProgress);
        let to_play = Move::new(Square::F2, Square::F3, &game.position);
        should_generate(&game.generate_all_legal_moves(), &to_play);
        game.play(&to_play);
        assert_eq!(game.position.state, State::Timeout);
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

    #[test]
    fn num_attackers() {
        let fen = "kr2r3/pp6/8/2N5/4pK2/8/2B1R1B1/8 w - - 0 1";
        let game = Game::from_position(Board::from_fen(fen).unwrap());
        let black_pawnbb = Square::E4;
        assert_eq!(game.num_attackers(black_pawnbb), 5);
    }
}
