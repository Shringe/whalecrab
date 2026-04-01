use std::{
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use crate::{
    bitboard::{BitBoard, EMPTY},
    castling::CastlingRights,
    file::File,
    movegen::{
        moves::Move,
        pieces::piece::{ALL_PIECE_TYPES, PieceColor, PieceType},
    },
    rank::Rank,
    square::Square,
};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

macro_rules! color_field_getters {
    ($field_name:ident, $return_type:ty) => {
        paste::paste! {
            pub fn [<get_ $field_name _mut>](&mut self, color: &crate::movegen::pieces::piece::PieceColor) -> &mut $return_type {
                match color {
                    crate::movegen::pieces::piece::PieceColor::White => &mut self.[<white_ $field_name>],
                    crate::movegen::pieces::piece::PieceColor::Black => &mut self.[<black_ $field_name>],
                }
            }

            pub fn [<get_ $field_name>](&self, color: &crate::movegen::pieces::piece::PieceColor) -> &$return_type {
                match color {
                    crate::movegen::pieces::piece::PieceColor::White => &self.[<white_ $field_name>],
                    crate::movegen::pieces::piece::PieceColor::Black => &self.[<black_ $field_name>],
                }
            }
        }
    };
}
pub(crate) use color_field_getters;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    InProgress,
    Checkmate,
    Stalemate,
    Timeout,
    Repetition,
}

/// Non-restoreable information needed to undo a move
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnRestoreable {
    pub castling_rights: CastlingRights,
    pub half_move_timeout: usize,
    pub en_passant_target: Option<Square>,
    // Not technically necessary but probably much faster to remember
    pub state: State,
}

#[derive(Clone)]
pub struct Game {
    // Piece bitboards (formerly Board fields)
    pub white_pawns: BitBoard,
    pub white_knights: BitBoard,
    pub white_bishops: BitBoard,
    pub white_rooks: BitBoard,
    pub white_queens: BitBoard,
    pub white_kings: BitBoard,

    pub black_pawns: BitBoard,
    pub black_knights: BitBoard,
    pub black_bishops: BitBoard,
    pub black_rooks: BitBoard,
    pub black_queens: BitBoard,
    pub black_kings: BitBoard,

    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<Square>,
    pub turn: PieceColor,

    pub half_move_timeout: usize,
    pub full_move_clock: usize,
    pub state: State,
    pub seen_positions: HashMap<u64, u8>,
    pub hash: u64,

    // Cached game state
    pub white_occupied: BitBoard,
    pub black_occupied: BitBoard,
    pub occupied: BitBoard,

    pub position_history: Vec<UnRestoreable>,
    pub white_attacks: BitBoard,
    pub black_attacks: BitBoard,
    pub white_check_rays: BitBoard,
    pub black_check_rays: BitBoard,
    pub legal_moves: Option<Vec<Move>>,
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Hash for Game {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.white_pawns.hash(state);
        self.white_knights.hash(state);
        self.white_bishops.hash(state);
        self.white_rooks.hash(state);
        self.white_queens.hash(state);
        self.white_kings.hash(state);
        self.black_pawns.hash(state);
        self.black_knights.hash(state);
        self.black_bishops.hash(state);
        self.black_rooks.hash(state);
        self.black_queens.hash(state);
        self.black_kings.hash(state);
        self.turn.hash(state);
        self.castling_rights.hash(state);
        self.en_passant_target.hash(state);
    }
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Self {
            white_pawns: BitBoard::INITIAL_WHITE_PAWN,
            white_knights: BitBoard::INITIAL_WHITE_KNIGHT,
            white_bishops: BitBoard::INITIAL_WHITE_BISHOP,
            white_rooks: BitBoard::INITIAL_WHITE_ROOK,
            white_queens: BitBoard::INITIAL_WHITE_QUEEN,
            white_kings: BitBoard::INITIAL_WHITE_KING,

            black_pawns: BitBoard::INITIAL_BLACK_PAWN,
            black_knights: BitBoard::INITIAL_BLACK_KNIGHT,
            black_bishops: BitBoard::INITIAL_BLACK_BISHOP,
            black_rooks: BitBoard::INITIAL_BLACK_ROOK,
            black_queens: BitBoard::INITIAL_BLACK_QUEEN,
            black_kings: BitBoard::INITIAL_BLACK_KING,

            castling_rights: CastlingRights::default(),
            en_passant_target: None,
            turn: PieceColor::White,

            half_move_timeout: 0,
            full_move_clock: 1,
            state: State::InProgress,
            seen_positions: HashMap::new(),
            hash: 0,

            white_attacks: EMPTY,
            black_attacks: EMPTY,
            white_check_rays: EMPTY,
            black_check_rays: EMPTY,
            white_occupied: EMPTY,
            black_occupied: EMPTY,
            occupied: EMPTY,
            position_history: Vec::new(),
            legal_moves: None,
        };

        game.refresh();
        game.seen_positions.insert(game.hash, 1);
        game
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Game(\"{}\")", self.to_fen())
    }
}

impl Game {
    color_field_getters!(attacks, BitBoard);
    color_field_getters!(check_rays, BitBoard);
    color_field_getters!(occupied, BitBoard);

    pub fn empty() -> Self {
        Self {
            white_pawns: EMPTY,
            white_knights: EMPTY,
            white_bishops: EMPTY,
            white_rooks: EMPTY,
            white_queens: EMPTY,
            white_kings: EMPTY,

            black_pawns: EMPTY,
            black_knights: EMPTY,
            black_bishops: EMPTY,
            black_rooks: EMPTY,
            black_queens: EMPTY,
            black_kings: EMPTY,

            castling_rights: CastlingRights::empty(),
            en_passant_target: None,
            turn: PieceColor::White,

            half_move_timeout: 0,
            full_move_clock: 0,
            state: State::InProgress,
            seen_positions: HashMap::new(),
            hash: 0,

            white_attacks: EMPTY,
            black_attacks: EMPTY,
            white_check_rays: EMPTY,
            black_check_rays: EMPTY,
            white_occupied: EMPTY,
            black_occupied: EMPTY,
            occupied: EMPTY,
            position_history: Vec::new(),
            legal_moves: None,
        }
    }

    /// Takes a fen string, parses and converts it into a game.
    /// Currently takes into account the following:
    /// - [x] Piece Placement
    /// - [x] Active Color
    /// - [x] Castling Rights
    /// - [x] En Passant Target
    /// - [ ] Halfmove Clock
    /// - [ ] Fullmove Number
    pub fn from_fen(fen: &str) -> Option<Self> {
        // Example Fen:
        // r1bqkbnr/ppp1pppp/2n5/1B1P4/8/8/PPPP1PPP/RNBQK1NR b KQkq - 2 3
        let mut split_fen = fen.split(' ');
        let body_fen = split_fen.next()?;
        let turn_fen = split_fen.next()?;
        let castling_fen = split_fen.next()?;
        let en_passant_fen = split_fen.next()?;
        let half_move_fen = split_fen.next()?;
        let full_move_fen = split_fen.next()?;

        let rows = body_fen.split('/');
        let mut game = Game::empty();

        for (rank, row) in rows.rev().enumerate() {
            let mut file = 0;
            for c in row.chars() {
                let sq = BitBoard::from_rank_file(Rank::from_index(rank), File::from_index(file));
                let colored_piece = match c {
                    'p' => Some((PieceType::Pawn, PieceColor::Black)),
                    'n' => Some((PieceType::Knight, PieceColor::Black)),
                    'b' => Some((PieceType::Bishop, PieceColor::Black)),
                    'r' => Some((PieceType::Rook, PieceColor::Black)),
                    'q' => Some((PieceType::Queen, PieceColor::Black)),
                    'k' => Some((PieceType::King, PieceColor::Black)),
                    'P' => Some((PieceType::Pawn, PieceColor::White)),
                    'N' => Some((PieceType::Knight, PieceColor::White)),
                    'B' => Some((PieceType::Bishop, PieceColor::White)),
                    'R' => Some((PieceType::Rook, PieceColor::White)),
                    'Q' => Some((PieceType::Queen, PieceColor::White)),
                    'K' => Some((PieceType::King, PieceColor::White)),
                    _ => None,
                };

                if let Some((piece, color)) = colored_piece {
                    game.set_occupied_bitboard(
                        &piece,
                        &color,
                        game.get_occupied_bitboard(&piece, &color) | sq,
                    );

                    file += 1;
                } else {
                    file += c.to_digit(10)? as usize;
                }
            }
        }

        game.turn = if turn_fen == "b" {
            PieceColor::Black
        } else {
            PieceColor::White
        };

        if castling_fen != "-" {
            game.castling_rights = CastlingRights {
                white_queenside: castling_fen.contains('Q'),
                white_kingside: castling_fen.contains('K'),
                black_queenside: castling_fen.contains('q'),
                black_kingside: castling_fen.contains('k'),
            }
        }

        if let Ok(sq) = Square::from_str(en_passant_fen) {
            game.en_passant_target = Some(sq);
        }

        if let Ok(half_moves) = half_move_fen.parse() {
            game.half_move_timeout = half_moves;
        }

        if let Ok(full_moves) = full_move_fen.parse() {
            game.full_move_clock = full_moves;
        }

        game.refresh();
        game.seen_positions.insert(game.hash, 1);

        Some(game)
    }

    /// Attempts to generate a fen from the current game state
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count: u8 = 0;

            for file in 0..8 {
                let square = Square::make_square(Rank::from_index(rank), File::from_index(file));
                let sqbb = BitBoard::from_square(square);

                if let Some((piece, color)) = self.determine_piece(&sqbb) {
                    // If we had empty squares, add the count first
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    let piece_char = match color {
                        PieceColor::White => match piece {
                            PieceType::Pawn => 'P',
                            PieceType::Knight => 'N',
                            PieceType::Bishop => 'B',
                            PieceType::Rook => 'R',
                            PieceType::Queen => 'Q',
                            PieceType::King => 'K',
                        },

                        PieceColor::Black => match piece {
                            PieceType::Pawn => 'p',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Rook => 'r',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                        },
                    };

                    fen.push(piece_char);
                } else {
                    empty_count += 1;
                }
            }

            // Add any remaining empty squares at the end of the rank
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            // Add rank separator (except for the last rank)
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(match self.turn {
            PieceColor::White => 'w',
            PieceColor::Black => 'b',
        });

        fen.push(' ');
        let mut castling = String::new();
        if self.castling_rights.white_kingside {
            castling.push('K');
        }
        if self.castling_rights.white_queenside {
            castling.push('Q');
        }
        if self.castling_rights.black_kingside {
            castling.push('k');
        }
        if self.castling_rights.black_queenside {
            castling.push('q');
        }

        if castling.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling);
        }

        fen.push(' ');
        if let Some(target) = self.en_passant_target {
            fen.push_str(&target.to_string().to_lowercase());
        } else {
            fen.push('-');
        }

        // Placeholder as the Halfmove and Fullmove clock are not implemented
        fen.push_str(format!(" {} {}", self.half_move_timeout, self.full_move_clock).as_str());

        fen
    }

    pub fn set_occupied_bitboard(&mut self, piece: &PieceType, color: &PieceColor, new: BitBoard) {
        match color {
            PieceColor::White => match piece {
                PieceType::Pawn => self.white_pawns = new,
                PieceType::Knight => self.white_knights = new,
                PieceType::Bishop => self.white_bishops = new,
                PieceType::Rook => self.white_rooks = new,
                PieceType::Queen => self.white_queens = new,
                PieceType::King => self.white_kings = new,
            },
            PieceColor::Black => match piece {
                PieceType::Pawn => self.black_pawns = new,
                PieceType::Knight => self.black_knights = new,
                PieceType::Bishop => self.black_bishops = new,
                PieceType::Rook => self.black_rooks = new,
                PieceType::Queen => self.black_queens = new,
                PieceType::King => self.black_kings = new,
            },
        }
    }

    pub fn get_occupied_bitboard(&self, piece: &PieceType, color: &PieceColor) -> BitBoard {
        match color {
            PieceColor::White => match piece {
                PieceType::Pawn => self.white_pawns,
                PieceType::Knight => self.white_knights,
                PieceType::Bishop => self.white_bishops,
                PieceType::Rook => self.white_rooks,
                PieceType::Queen => self.white_queens,
                PieceType::King => self.white_kings,
            },
            PieceColor::Black => match piece {
                PieceType::Pawn => self.black_pawns,
                PieceType::Knight => self.black_knights,
                PieceType::Bishop => self.black_bishops,
                PieceType::Rook => self.black_rooks,
                PieceType::Queen => self.black_queens,
                PieceType::King => self.black_kings,
            },
        }
    }

    /// Restores the essential data from the previous position
    pub fn restore_position(&mut self) {
        let last_position = self
            .position_history
            .pop()
            .expect("Tried to unmake a move, but the required information is not present");
        self.castling_rights = last_position.castling_rights;
        self.half_move_timeout = last_position.half_move_timeout;
        self.en_passant_target = last_position.en_passant_target;
        self.state = last_position.state;
    }

    /// Captures essential position information to be restored later
    pub fn capture_position(&mut self) {
        let last_position = UnRestoreable {
            castling_rights: self.castling_rights,
            half_move_timeout: self.half_move_timeout,
            en_passant_target: self.en_passant_target,
            state: self.state,
        };
        self.position_history.push(last_position);
    }

    /// Recalculates certain cached values regarding the position
    /// Should be called on Self initialization and position updates
    fn refresh(&mut self) {
        let white_pieces = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_kings;
        let black_pieces = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_kings;
        let pieces = white_pieces | black_pieces;

        debug_assert_eq!(
            white_pieces & black_pieces,
            EMPTY,
            "Both white and black claim to have pieces on the same square"
        );

        self.white_occupied = white_pieces;
        self.black_occupied = black_pieces;
        self.occupied = pieces;

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        self.hash = hasher.finish();

        self.update_attacks();
    }

    /// Calculates the attack bitboard for the given player
    fn calculate_attacks(&self, color: &PieceColor) -> (BitBoard, BitBoard) {
        let mut attacks = EMPTY;
        let mut check_rays = EMPTY;

        for sq in *self.get_occupied(color) {
            let sqbb = BitBoard::from_square(sq);
            let (piece, _) = self.determine_piece(&sqbb).unwrap();
            let moveinfo = piece.psuedo_legal_targets_fast(self, &sq);
            attacks |= moveinfo.attacks;
            check_rays |= moveinfo.check_rays;
        }

        (attacks, check_rays)
    }

    /// Updates attack bitboard for the both players
    fn update_attacks(&mut self) {
        let color = self.turn;
        let enemy = color.opponent();

        (
            *self.get_attacks_mut(&color),
            *self.get_check_rays_mut(&color),
        ) = self.calculate_attacks(&color);

        (
            *self.get_attacks_mut(&enemy),
            *self.get_check_rays_mut(&enemy),
        ) = self.calculate_attacks(&enemy);

        if self.state == State::InProgress {
            let moves = self.generate_all_legal_moves();

            if moves.is_empty() {
                self.state = if self.is_in_check() {
                    State::Checkmate
                } else {
                    State::Stalemate
                };
            }

            self.legal_moves = Some(moves);
        }
    }

    /// Checks if the current player's king is in check
    pub fn is_in_check(&self) -> bool {
        match self.turn {
            PieceColor::White => self.black_attacks.has_square(&self.white_kings),
            PieceColor::Black => self.white_attacks.has_square(&self.black_kings),
        }
    }

    /// Returns a bitboard of every piece attacking the given square
    pub fn attackers(&self, sq: Square) -> BitBoard {
        let mut attackers = EMPTY;
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
            let moveinfo = piece.psuedo_legal_targets_fast(self, &sq);
            let potential_enemy = self.get_pieces(&piece, &enemy);
            attackers |= moveinfo.attacks & potential_enemy;
        }

        attackers
    }

    /// Finishes a turn and determines game state is possible
    pub fn next_turn(&mut self, last_move: &Move) {
        // Handle en_passant
        self.en_passant_target = match last_move {
            Move::CreateEnPassant { at } => match self.turn {
                PieceColor::White => Some(Square::make_square(Rank::Third, *at)),
                PieceColor::Black => Some(Square::make_square(Rank::Sixth, *at)),
            },
            _ => None,
        };

        // Update position state
        self.turn = self.turn.opponent();
        if self.turn == PieceColor::White {
            self.full_move_clock += 1;
        }
        self.refresh();

        // Half move timeout
        let should_reset_half_move_timeout = match last_move {
            Move::Normal { to, capture, .. } => {
                capture.is_some()
                    || matches!(
                        self.determine_piece(&BitBoard::from_square(*to)),
                        Some((PieceType::Pawn, _))
                    )
            }
            Move::CreateEnPassant { .. } => true,
            Move::CaptureEnPassant { .. } => true,
            Move::Promotion { .. } => true,
            Move::Castle { .. } => false,
        };

        if should_reset_half_move_timeout {
            self.half_move_timeout = 0;
        } else {
            self.half_move_timeout += 1;
        }

        if self.half_move_timeout == 50 {
            self.state = State::Timeout;
        }

        // Repetition
        if let Some(times_seen) = self.seen_positions.get_mut(&self.hash) {
            if *times_seen == 2 {
                self.state = State::Repetition;
            }
            *times_seen += 1;
        } else {
            self.seen_positions.insert(self.hash, 1);
        }
    }

    /// Reverses turn color and full_move_clock to the last turn
    pub fn previous_turn(&mut self) {
        // Repetition
        if let Some(times_seen) = self.seen_positions.get_mut(&self.hash) {
            if *times_seen == 1 {
                self.seen_positions.remove(&self.hash);
            } else {
                *times_seen -= 1;
            }
        }

        self.turn = self.turn.opponent();

        self.refresh();
        if self.turn == PieceColor::Black {
            self.full_move_clock -= 1;
        }
    }

    /// Gets the bitboard of a colored piece
    pub fn get_pieces(&self, piece: &PieceType, color: &PieceColor) -> &BitBoard {
        match color {
            PieceColor::White => match piece {
                PieceType::Pawn => &self.white_pawns,
                PieceType::Knight => &self.white_knights,
                PieceType::Bishop => &self.white_bishops,
                PieceType::Rook => &self.white_rooks,
                PieceType::Queen => &self.white_queens,
                PieceType::King => &self.white_kings,
            },
            PieceColor::Black => match piece {
                PieceType::Pawn => &self.black_pawns,
                PieceType::Knight => &self.black_knights,
                PieceType::Bishop => &self.black_bishops,
                PieceType::Rook => &self.black_rooks,
                PieceType::Queen => &self.black_queens,
                PieceType::King => &self.black_kings,
            },
        }
    }

    /// Gets the bitboard of a colored piece
    pub fn get_pieces_mut(&mut self, piece: &PieceType, color: &PieceColor) -> &mut BitBoard {
        match color {
            PieceColor::White => match piece {
                PieceType::Pawn => &mut self.white_pawns,
                PieceType::Knight => &mut self.white_knights,
                PieceType::Bishop => &mut self.white_bishops,
                PieceType::Rook => &mut self.white_rooks,
                PieceType::Queen => &mut self.white_queens,
                PieceType::King => &mut self.white_kings,
            },
            PieceColor::Black => match piece {
                PieceType::Pawn => &mut self.black_pawns,
                PieceType::Knight => &mut self.black_knights,
                PieceType::Bishop => &mut self.black_bishops,
                PieceType::Rook => &mut self.black_rooks,
                PieceType::Queen => &mut self.black_queens,
                PieceType::King => &mut self.black_kings,
            },
        }
    }

    /// Determines color of standing piece
    pub fn determine_color(&self, sqbb: &BitBoard) -> Option<PieceColor> {
        if self.white_occupied.has_square(sqbb) {
            Some(PieceColor::White)
        } else if self.black_occupied.has_square(sqbb) {
            Some(PieceColor::Black)
        } else {
            None
        }
    }

    /// Determines type and color of standing piece
    pub fn determine_piece(&self, sqbb: &BitBoard) -> Option<(PieceType, PieceColor)> {
        if self.white_occupied.has_square(sqbb) {
            if self.white_pawns.has_square(sqbb) {
                Some((PieceType::Pawn, PieceColor::White))
            } else if self.white_knights.has_square(sqbb) {
                Some((PieceType::Knight, PieceColor::White))
            } else if self.white_bishops.has_square(sqbb) {
                Some((PieceType::Bishop, PieceColor::White))
            } else if self.white_rooks.has_square(sqbb) {
                Some((PieceType::Rook, PieceColor::White))
            } else if self.white_queens.has_square(sqbb) {
                Some((PieceType::Queen, PieceColor::White))
            } else if self.white_kings.has_square(sqbb) {
                Some((PieceType::King, PieceColor::White))
            } else {
                unreachable!("The white occupied bitboard has a square that no white pieces have!")
            }
        } else if self.black_occupied.has_square(sqbb) {
            if self.black_pawns.has_square(sqbb) {
                Some((PieceType::Pawn, PieceColor::Black))
            } else if self.black_knights.has_square(sqbb) {
                Some((PieceType::Knight, PieceColor::Black))
            } else if self.black_bishops.has_square(sqbb) {
                Some((PieceType::Bishop, PieceColor::Black))
            } else if self.black_rooks.has_square(sqbb) {
                Some((PieceType::Rook, PieceColor::Black))
            } else if self.black_queens.has_square(sqbb) {
                Some((PieceType::Queen, PieceColor::Black))
            } else if self.black_kings.has_square(sqbb) {
                Some((PieceType::King, PieceColor::Black))
            } else {
                unreachable!("The black occupied bitboard has a square that no black pieces have!")
            }
        } else {
            None
        }
    }

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let color = &self.turn;
        let occupied = self.get_occupied(color);

        for sq in *occupied {
            let sqbb = BitBoard::from_square(sq);
            if let Some((piece, _)) = self.determine_piece(&sqbb) {
                let moveinfo = piece.psuedo_legal_targets_fast(self, &sq);
                for t in moveinfo.targets {
                    moves.push(Move::infer(sq, t, self));
                }
            }
        }

        moves
    }

    /// Hands over pregenerated legal moves on the first call, and generates legal moves
    /// again for each subsequent call. If you want to call this method multiple times,
    /// think about calling this method once and storing the output instead.
    pub fn legal_moves(&mut self) -> Vec<Move> {
        if self.state != State::InProgress {
            return Vec::new();
        }

        match self.legal_moves.take() {
            Some(legal_moves) => legal_moves,
            None => self.generate_all_legal_moves(),
        }
    }

    /// Generates all legal moves for the current player. This also updates position state
    /// for statemate or checkmate
    fn generate_all_legal_moves(&self) -> Vec<Move> {
        self.legal_moves_filter(self.generate_all_psuedo_legal_moves())
    }
}

#[cfg(test)]
mod tests {
    use crate::bitboard::{BitBoard, EMPTY};
    use crate::game::Game;
    use crate::game::{STARTING_FEN, State};
    use crate::movegen::moves::Move;
    use crate::movegen::pieces::piece::{PieceColor, PieceType};
    use crate::square::Square;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

    #[test]
    fn white_gets_checkmated() {
        let fen = "2r5/8/8/8/8/8/5k2/7K w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.state, State::InProgress);
        let moves = [(Square::H1, Square::H2), (Square::C8, Square::H8)];
        for (from, to) in moves {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        game.legal_moves();
        assert_eq!(game.turn, PieceColor::White);
        assert_eq!(game.state, State::Checkmate);
    }

    #[test]
    fn black_gets_stalemated() {
        let fen = "4k3/4P3/5K2/8/8/8/8/8 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let to_play = Move::infer(Square::F6, Square::E6, &game);

        assert_eq!(game.state, State::InProgress);
        should_generate(&game.legal_moves(), &to_play);
        game.play(&to_play);
        let moves = game.legal_moves();
        println!(
            "white attacks:\n{}\nblack attacks:\n{}",
            game.white_attacks, game.black_attacks
        );
        assert!(
            moves.is_empty(),
            "Black should not have any moves, but has: {}",
            format_pretty_list(&moves)
        );
        assert_eq!(game.turn, PieceColor::Black);
        assert_eq!(game.state, State::Stalemate);
    }

    #[test]
    fn draw_fifty_move_rule() {
        let fen = "4k3/8/8/8/8/8/1NNN1KN1/8 w - - 49 1";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.state, State::InProgress);
        let to_play = Move::infer(Square::F2, Square::F3, &game);
        should_generate(&game.legal_moves(), &to_play);
        game.play(&to_play);
        assert_eq!(game.state, State::Timeout);
    }

    #[test]
    fn draw_by_repetition() {
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
            // (Square::G1, Square::F3),
        ];

        for (from, to) in moves {
            assert_eq!(game.state, State::InProgress);
            let m = Move::infer(from, to, &game);
            should_generate(&game.legal_moves(), &m);
            game.play(&m);
        }

        assert_eq!(game.state, State::Repetition);
    }

    #[test]
    fn should_not_have_moves_after_draw_by_repetition() {
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
            // (Square::G1, Square::F3),
        ];

        for (from, to) in moves {
            assert_eq!(game.state, State::InProgress);
            let m = Move::infer(from, to, &game);
            should_generate(&game.legal_moves(), &m);
            game.play(&m);
        }

        let moves = game.legal_moves();
        println!("{:?}", game);
        assert_eq!(game.state, State::Repetition);
        assert!(moves.is_empty(), "{}", format_pretty_list(&moves));
    }

    #[test]
    fn num_attackers() {
        let fen = "kr2r3/pp6/8/2N5/4pK2/8/2B1R1B1/8 w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let black_pawnbb = Square::E4;
        assert_eq!(game.attackers(black_pawnbb).popcnt(), 5);
    }

    #[test]
    #[ignore]
    fn game_comes_to_an_end() {
        let mut game = Game::default();
        loop {
            let moves = game.legal_moves();
            let m = match moves.first() {
                Some(m) => m,
                None => break,
            };

            println!("Playing: {}", m);
            game.play(m);
        }

        let moves_left = game.legal_moves();

        assert_ne!(
            game.state,
            State::InProgress,
            "Fen: {}\n{:?} still had {}",
            game.to_fen(),
            game.turn,
            format_pretty_list(&moves_left)
        );
    }

    #[test]
    fn can_capture_attacking_rook() {
        let fen = "rR1k3r/2p3p1/p1P2p1p/2Bpp3/8/6P1/P6P/1R4K1 b - - 3 33";
        let mut game = Game::from_fen(fen).unwrap();
        let moves = game.legal_moves();
        should_generate(
            &moves,
            &Move::Normal {
                from: Square::A8,
                to: Square::B8,
                capture: Some(PieceType::Rook),
            },
        );
    }

    #[test]
    fn to_fen() {
        let fen_before = "rnbq1rk1/p1p2p1p/3bpp2/1p6/2pP4/2N1B3/PP1Q1PPP/R3KBNR w KQ - 4 9";
        let game_before = Game::from_fen(fen_before).unwrap();

        let fen_after = game_before.to_fen();
        let game_after = Game::from_fen(&fen_after).unwrap();

        // I'm comparing the two boards directly instead of their fen because some parts of the fen implemenation
        // are not yet implemented, which would fail the test.
        assert_eq!(game_before, game_after);
    }

    #[test]
    fn en_passant_fen() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::E2, Square::E4),
            (Square::D7, Square::D5),
            (Square::E4, Square::E5),
            (Square::F7, Square::F5),
        ] {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
        compare_to_fen(&game, fen);
    }

    #[test]
    fn from_fen_considers_en_passant_target() {
        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 2";
        let game = Game::from_fen(fen).unwrap();
        assert_eq!(game.en_passant_target, Some(Square::F6));
    }

    #[test]
    fn starting_fen() {
        let game = Game::default();
        compare_to_fen(&game, STARTING_FEN);
    }

    #[test]
    fn complex_fen() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::E2, Square::E4),
            (Square::D7, Square::D5),
            (Square::E4, Square::D5),
            (Square::B8, Square::C6),
            (Square::F1, Square::B5),
        ] {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        let fen = "r1bqkbnr/ppp1pppp/2n5/1B1P4/8/8/PPPP1PPP/RNBQK1NR b KQkq - 2 3";
        compare_to_fen(&game, fen);
    }

    #[test]
    fn determine_colors() {
        let game = Game::default();

        let white = Square::C2;
        let empty = Square::G4;
        let black = Square::B8;
        let queen = Square::D1;

        assert_eq!(
            game.determine_color(&BitBoard::from_square(white)),
            Some(PieceColor::White)
        );
        assert_eq!(game.determine_color(&BitBoard::from_square(empty)), None);
        assert_eq!(
            game.determine_color(&BitBoard::from_square(black)),
            Some(PieceColor::Black)
        );
        assert_eq!(
            game.determine_color(&BitBoard::from_square(queen)),
            Some(PieceColor::White)
        );
    }

    #[test]
    fn determine_pieces() {
        let game = Game::default();

        let pawn = Square::C2;
        let empty = Square::G4;
        let knight = Square::B8;
        let queen = Square::D8;

        assert_eq!(
            game.determine_piece(&BitBoard::from_square(pawn))
                .map(|(p, _)| p),
            Some(PieceType::Pawn)
        );
        assert_eq!(game.determine_piece(&BitBoard::from_square(empty)), None);
        assert_eq!(
            game.determine_piece(&BitBoard::from_square(knight))
                .map(|(p, _)| p),
            Some(PieceType::Knight)
        );
        assert_eq!(
            game.determine_piece(&BitBoard::from_square(queen))
                .map(|(p, _)| p),
            Some(PieceType::Queen)
        );
    }

    #[test]
    fn get_occupied_bitboards() {
        let game = Game::default();

        let white_pawns = game.get_occupied_bitboard(&PieceType::Pawn, &PieceColor::White);
        assert_eq!(white_pawns, game.white_pawns);
        assert!(BitBoard::from_square(Square::A2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::H2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::A3) & white_pawns == EMPTY);
        assert!(BitBoard::from_square(Square::E4) & white_pawns == EMPTY);

        let black_rooks = game.get_occupied_bitboard(&PieceType::Rook, &PieceColor::Black);
        assert_eq!(black_rooks, game.black_rooks);
        assert!(BitBoard::from_square(Square::A8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::H8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::B7) & black_rooks == EMPTY);
        assert!(BitBoard::from_square(Square::E5) & black_rooks == EMPTY);
    }

    #[test]
    fn hash_determinism() {
        let fen_after = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let mut game = Game::default();
        game.play(&Move::infer(Square::E2, Square::E4, &game));
        let fen_game = Game::from_fen(fen_after).unwrap();
        let mut game_after_refresh = game.clone();
        let mut fen_game_after_refresh = fen_game.clone();
        game_after_refresh.refresh();
        fen_game_after_refresh.refresh();

        assert_eq!(game.en_passant_target, fen_game.en_passant_target);

        assert_eq!(
            game_after_refresh.hash, fen_game_after_refresh.hash,
            "The naturally reached position has a different hash than the one generated from fen, even after refreshing them both"
        );
        assert_eq!(
            game.hash, fen_game.hash,
            "The naturally reached position has a different hash than the one generated from fen"
        );
    }

    #[test]
    fn generating_legal_moves_should_not_mutate_position() {
        let mut game = Game::default();
        let mut last_moves = game.legal_moves();
        let mut last = game.clone();
        for _ in 1..20 {
            let moves = game.legal_moves();
            let game = game.clone();
            assert_eq!(moves, last_moves);
            assert_eq!(game, last);
            last = game;
            last_moves = moves;
        }
    }
}
