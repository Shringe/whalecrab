use std::{
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

#[cfg(feature = "panic_logger")]
use panic_logger::BufLogger;

use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    get_attacks, get_attacks_mut, get_check_rays, get_check_rays_mut, get_occupied,
    get_occupied_mut, get_pieces, get_pieces_mut,
    movegen::{
        moves::Move,
        pieces::piece::{ALL_PIECE_TYPES, PieceColor, PieceType},
    },
    position::{
        castling::CastlingRights,
        piece_table::PieceTable,
        previous::{PositionHistory, UnRestoreable},
    },
    rank::Rank,
    square::Square,
};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    InProgress,
    Checkmate,
    Stalemate,
    Timeout,
    Repetition,
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

    pub half_move_timeout: u8,
    pub full_move_clock: u16,
    pub state: State,
    pub seen_positions: HashMap<u64, u8>,
    pub hash: u64,

    // Cached game state
    pub white_occupied: BitBoard,
    pub black_occupied: BitBoard,
    pub occupied: BitBoard,

    position_history: PositionHistory,
    pub white_attacks: BitBoard,
    pub black_attacks: BitBoard,
    pub white_check_rays: BitBoard,
    pub black_check_rays: BitBoard,
    pub legal_moves: Option<Vec<Move>>,
    pub(crate) piece_table: PieceTable,
    #[cfg(feature = "panic_logger")]
    panic_logger: BufLogger,
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
            position_history: PositionHistory::new(),
            legal_moves: None,
            piece_table: PieceTable::new(),
            #[cfg(feature = "panic_logger")]
            panic_logger: BufLogger::new(),
        };

        game.initialize();
        game
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return write!(f, "Game(\"{}\")", self.to_fen());
        }

        let separator = "=".repeat(20);
        let divider = "-".repeat(20);

        let mut out = String::new();
        out.push('\n');

        out.push_str(&format!("{separator} BitBoards {separator}\n"));
        out.push_str(&format!("White pawns:\n{}\n", self.white_pawns));
        out.push_str(&format!("White knights:\n{}\n", self.white_knights));
        out.push_str(&format!("White bishops:\n{}\n", self.white_bishops));
        out.push_str(&format!("White rooks:\n{}\n", self.white_rooks));
        out.push_str(&format!("White queens:\n{}\n", self.white_queens));
        out.push_str(&format!("White kings:\n{}\n", self.white_kings));
        out.push_str(&format!("{divider}\n"));
        out.push_str(&format!("Black pawns:\n{}\n", self.black_pawns));
        out.push_str(&format!("Black knights:\n{}\n", self.black_knights));
        out.push_str(&format!("Black bishops:\n{}\n", self.black_bishops));
        out.push_str(&format!("Black rooks:\n{}\n", self.black_rooks));
        out.push_str(&format!("Black queens:\n{}\n", self.black_queens));
        out.push_str(&format!("Black kings:\n{}\n", self.black_kings));

        out.push_str(&format!("{separator} Attacks {separator}\n"));
        out.push_str(&format!("White attacks:\n{}\n", self.white_attacks));
        out.push_str(&format!("Black attacks:\n{}\n", self.black_attacks));

        out.push_str(&format!("{separator} Check Rays {separator}\n"));
        out.push_str(&format!("White check rays:\n{}\n", self.white_check_rays));
        out.push_str(&format!("Black check rays:\n{}\n", self.black_check_rays));

        out.push_str(&format!("{separator} State {separator}\n"));
        out.push_str(&format!("Turn:              {:?}\n", self.turn));
        out.push_str(&format!("State:             {:?}\n", self.state));
        out.push_str(&format!("Castling rights:   {:#?}\n", self.castling_rights));
        out.push_str(&format!(
            "En passant target: {:?}\n",
            self.en_passant_target
        ));
        out.push_str(&format!("Half move clock:   {}\n", self.half_move_timeout));
        out.push_str(&format!("Full move clock:   {}\n", self.full_move_clock));
        out.push_str(&format!("Hash:              {:#018x}\n", self.hash));
        out.push_str(&format!("FEN:               {}\n", self.to_fen()));

        write!(f, "{}", out)
    }
}

impl Game {
    /// Pushes a log to the log buffer if cfg!(feature = "panic_logger")
    pub fn log<S: ToString>(&mut self, msg: S) {
        #[cfg(feature = "panic_logger")]
        self.panic_logger.push(msg.to_string());
    }

    /// Dumps the recent logs to stderr if cfg!(feature = "panic_logger")
    pub fn dump_logs(&self) {
        #[cfg(feature = "panic_logger")]
        {
            let logs = self.panic_logger.retrieve();
            eprintln!("Recent logs:\n{}", logs);
        }
    }

    // Piece getters
    pub fn get_attacks(&self, color: &PieceColor) -> &BitBoard {
        get_attacks!(self, color)
    }
    pub fn get_attacks_mut(&mut self, color: &PieceColor) -> &mut BitBoard {
        get_attacks_mut!(self, color)
    }
    pub fn get_check_rays(&self, color: &PieceColor) -> &BitBoard {
        get_check_rays!(self, color)
    }
    pub fn get_check_rays_mut(&mut self, color: &PieceColor) -> &mut BitBoard {
        get_check_rays_mut!(self, color)
    }
    pub fn get_occupied(&self, color: &PieceColor) -> &BitBoard {
        get_occupied!(self, color)
    }
    pub fn get_occupied_mut(&mut self, color: &PieceColor) -> &mut BitBoard {
        get_occupied_mut!(self, color)
    }
    /// Gets the bitboard of a colored piece
    pub fn get_pieces(&self, piece: &PieceType, color: &PieceColor) -> &BitBoard {
        get_pieces!(self, piece, color)
    }
    /// Gets the bitboard of a colored piece
    pub fn get_pieces_mut(&mut self, piece: &PieceType, color: &PieceColor) -> &mut BitBoard {
        get_pieces_mut!(self, piece, color)
    }

    // Constructors
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
            position_history: PositionHistory::new(),
            legal_moves: None,
            piece_table: PieceTable::new(),
            #[cfg(feature = "panic_logger")]
            panic_logger: BufLogger::new(),
        }
    }

    /// Takes a fen string, parses and converts it into a game.
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
                let sqbb = BitBoard::from_rank_file(Rank::from_index(rank), File::from_index(file));
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
                    let pieces = game.get_pieces_mut(&piece, &color);
                    *pieces |= sqbb;
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
            game.castling_rights = CastlingRights::from_fen(castling_fen);
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

        game.initialize();

        Some(game)
    }

    /// Attempts to generate a fen from the current game state
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count: u8 = 0;

            for file in 0..8 {
                let sq = Square::make_square(Rank::from_index(rank), File::from_index(file));

                if let Some((piece, color)) = self.piece_lookup(sq) {
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
        fen.push_str(&self.castling_rights.to_fen());

        fen.push(' ');
        if let Some(target) = self.en_passant_target {
            fen.push_str(&target.to_string().to_lowercase());
        } else {
            fen.push('-');
        }

        fen.push_str(format!(" {} {}", self.half_move_timeout, self.full_move_clock).as_str());

        fen
    }

    // Move generation related
    /// Restores the essential data from the previous position
    pub(crate) fn restore_position(&mut self) {
        let last_position = self
            .position_history
            .pop(self.turn)
            .expect("Tried to unmake a move, but the required information is not present");
        self.castling_rights = last_position.castling_rights;
        self.half_move_timeout = last_position.half_move_timeout;
        self.en_passant_target = last_position.en_passant_target;
        // We can assume that this position was reached from a non-terminal state
        self.state = State::InProgress;
    }

    /// Captures essential position information to be restored later
    pub(crate) fn capture_position(&mut self) {
        let last_position = UnRestoreable {
            castling_rights: self.castling_rights,
            half_move_timeout: self.half_move_timeout,
            en_passant_target: self.en_passant_target,
        };
        self.position_history.push(last_position);
    }

    /// Finishes a turn and determines game state is possible
    pub(crate) fn next_turn(&mut self, last_move: &Move) {
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
        if let Some(moves) = &self.legal_moves
            && moves.is_empty()
        {
            self.state = if self.is_in_check() {
                State::Checkmate
            } else {
                State::Stalemate
            };
        }

        // Half move timeout
        let should_reset_half_move_timeout = match last_move {
            Move::Normal { to, capture, .. } => {
                capture.is_some() || matches!(self.piece_lookup(*to), Some((PieceType::Pawn, _)))
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
    pub(crate) fn previous_turn(&mut self) {
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

    // Game initializers
    /// Initalizes the game. This should only be called inside of constructors
    fn initialize(&mut self) {
        self.populate_piece_table();
        self.refresh();
        self.seen_positions.insert(self.hash, 1);
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
        let moves = self.generate_all_legal_moves();
        self.legal_moves = Some(moves);
    }

    /// Calculates the attack bitboard for the given player
    fn calculate_attacks(&self, color: &PieceColor) -> (BitBoard, BitBoard) {
        let mut attacks = EMPTY;
        let mut check_rays = EMPTY;

        for sq in *self.get_occupied(color) {
            let Some((piece, _)) = self.piece_lookup(sq) else {
                panic!(
                    "The piece lookup table has a fake piece! {:?}\n{:?}",
                    self,
                    self.get_occupied(color)
                )
            };
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
    }

    /// Fully recalculates the piece table
    fn populate_piece_table(&mut self) {
        // Clear the table first
        self.piece_table = PieceTable::new();

        for sq in self.white_pawns {
            self.piece_table
                .set(sq, Some((PieceType::Pawn, PieceColor::White)));
        }
        for sq in self.white_knights {
            self.piece_table
                .set(sq, Some((PieceType::Knight, PieceColor::White)));
        }
        for sq in self.white_bishops {
            self.piece_table
                .set(sq, Some((PieceType::Bishop, PieceColor::White)));
        }
        for sq in self.white_rooks {
            self.piece_table
                .set(sq, Some((PieceType::Rook, PieceColor::White)));
        }
        for sq in self.white_queens {
            self.piece_table
                .set(sq, Some((PieceType::Queen, PieceColor::White)));
        }
        for sq in self.white_kings {
            self.piece_table
                .set(sq, Some((PieceType::King, PieceColor::White)));
        }
        for sq in self.black_pawns {
            self.piece_table
                .set(sq, Some((PieceType::Pawn, PieceColor::Black)));
        }
        for sq in self.black_knights {
            self.piece_table
                .set(sq, Some((PieceType::Knight, PieceColor::Black)));
        }
        for sq in self.black_bishops {
            self.piece_table
                .set(sq, Some((PieceType::Bishop, PieceColor::Black)));
        }
        for sq in self.black_rooks {
            self.piece_table
                .set(sq, Some((PieceType::Rook, PieceColor::Black)));
        }
        for sq in self.black_queens {
            self.piece_table
                .set(sq, Some((PieceType::Queen, PieceColor::Black)));
        }
        for sq in self.black_kings {
            self.piece_table
                .set(sq, Some((PieceType::King, PieceColor::Black)));
        }
    }

    // Game/state queries
    /// Checks if the current player's king is in check
    pub fn is_in_check(&self) -> bool {
        match self.turn {
            PieceColor::White => self.black_attacks.has_square(self.white_kings),
            PieceColor::Black => self.white_attacks.has_square(self.black_kings),
        }
    }

    /// Returns a bitboard of every piece attacking the given square
    pub fn attackers(&self, sq: Square) -> BitBoard {
        let mut attackers = EMPTY;
        let sqbb = BitBoard::from_square(sq);
        let color = if let Some(color) = self.determine_color(sqbb) {
            color
        } else {
            return attackers;
        };

        let enemy = color.opponent();
        if !self.get_attacks(&enemy).has_square(sqbb) {
            return attackers;
        }

        for piece in ALL_PIECE_TYPES {
            let moveinfo = piece.psuedo_legal_targets_fast(self, &sq);
            let potential_enemy = self.get_pieces(&piece, &enemy);
            attackers |= moveinfo.attacks & *potential_enemy;
        }

        attackers
    }

    /// Determines color of standing piece.
    /// This is faster than calling Game.piece_lookup and then discarding the piece type.
    pub fn determine_color(&self, sqbb: BitBoard) -> Option<PieceColor> {
        if self.white_occupied.has_square(sqbb) {
            Some(PieceColor::White)
        } else if self.black_occupied.has_square(sqbb) {
            Some(PieceColor::Black)
        } else {
            None
        }
    }

    /// Gets the type and color of a potential piece on the given square
    pub fn piece_lookup(&self, sq: Square) -> Option<(PieceType, PieceColor)> {
        self.piece_table.get(sq)
    }

    /// # Safety
    /// The counter must be set to the current length of `moves`, `moves` must have
    /// enough capacity, and `moves.set_len(counter)` must be called after.
    pub unsafe fn generate_grouped_psuedo_legal_white_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        counter: &mut usize,
    ) {
        let twice_mask = Rank::Fourth.mask();
        let promotion_mask = Rank::Eighth.mask();
        let unoccupied = !self.occupied;

        let once = self.white_pawns.up() & unoccupied;
        let twice = once.up() & unoccupied & twice_mask;
        let promotions = once & promotion_mask;

        let capture_right = self.white_pawns.up_right() & (self.black_occupied & !File::A.mask());
        let capture_left = self.white_pawns.up_left() & (self.black_occupied & !File::H.mask());

        macro_rules! get_piece {
            ($sq:expr) => {
                Some(
                    if cfg!(debug_assertions) {
                        self.piece_lookup($sq).unwrap()
                    } else {
                        // Should be safe because with pawn move generation we know for sure
                        // whether or not we can capture ahead of time using bit manipulation
                        unsafe { self.piece_lookup($sq).unwrap_unchecked() }
                    }
                    .0,
                )
            };
        }

        for to in once ^ promotions {
            let from = unsafe { to.down_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: None,
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for sq in twice {
            let m = Move::CreateEnPassant { at: sq.get_file() };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for sq in promotions {
            let file = sq.get_file();
            let m = Move::Promotion {
                from: file,
                to: file,
                piece: PieceType::Queen,
                capture: None,
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_right & !promotion_mask {
            let from = unsafe { to.dleft_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_left & !promotion_mask {
            let from = unsafe { to.dright_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_right & promotion_mask {
            let from = unsafe { to.dleft_unchecked() };
            let m = Move::Promotion {
                from: from.get_file(),
                to: to.get_file(),
                piece: PieceType::Queen,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_left & promotion_mask {
            let from = unsafe { to.dright_unchecked() };
            let m = Move::Promotion {
                from: from.get_file(),
                to: to.get_file(),
                piece: PieceType::Queen,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        if let Some(target) = self.en_passant_target {
            let mut process_target = |sq: Option<Square>| {
                if let Some(sq) = sq
                    && self.white_pawns.has_square(BitBoard::from_square(sq))
                {
                    let m = Move::CaptureEnPassant {
                        from: sq.get_file(),
                    };
                    unsafe {
                        push_move_unchecked(moves, m, counter);
                    }
                }
            };
            process_target(target.dleft());
            process_target(target.dright());
        }
    }

    /// # Safety
    /// The counter must be set to the current length of `moves`, `moves` must have
    /// enough capacity, and `moves.set_len(counter)` must be called after.
    pub unsafe fn generate_grouped_psuedo_legal_black_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        counter: &mut usize,
    ) {
        let twice_mask = Rank::Fifth.mask();
        let promotion_mask = Rank::First.mask();
        let unoccupied = !self.occupied;

        let once = self.black_pawns.down() & unoccupied;
        let twice = once.down() & unoccupied & twice_mask;
        let promotions = once & promotion_mask;

        let capture_right = self.black_pawns.down_left() & (self.white_occupied & !File::H.mask());
        let capture_left = self.black_pawns.down_right() & (self.white_occupied & !File::A.mask());

        macro_rules! get_piece {
            ($sq:expr) => {
                Some(
                    if cfg!(debug_assertions) {
                        self.piece_lookup($sq).unwrap()
                    } else {
                        unsafe { self.piece_lookup($sq).unwrap_unchecked() }
                    }
                    .0,
                )
            };
        }

        for to in once ^ promotions {
            let from = unsafe { to.up_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: None,
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for sq in twice {
            let m = Move::CreateEnPassant { at: sq.get_file() };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for sq in promotions {
            let file = sq.get_file();
            let m = Move::Promotion {
                from: file,
                to: file,
                piece: PieceType::Queen,
                capture: None,
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_right & !promotion_mask {
            let from = unsafe { to.uright_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_left & !promotion_mask {
            let from = unsafe { to.uleft_unchecked() };
            let m = Move::Normal {
                from,
                to,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_right & promotion_mask {
            let from = unsafe { to.uleft_unchecked() };
            let m = Move::Promotion {
                from: from.get_file(),
                to: to.get_file(),
                piece: PieceType::Queen,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        for to in capture_left & promotion_mask {
            let from = unsafe { to.uright_unchecked() };
            let m = Move::Promotion {
                from: from.get_file(),
                to: to.get_file(),
                piece: PieceType::Queen,
                capture: get_piece!(to),
            };
            unsafe {
                push_move_unchecked(moves, m, counter);
            }
        }

        if let Some(target) = self.en_passant_target {
            let mut process_target = |sq: Option<Square>| {
                if let Some(sq) = sq
                    && self.black_pawns.has_square(BitBoard::from_square(sq))
                {
                    let m = Move::CaptureEnPassant {
                        from: sq.get_file(),
                    };
                    unsafe {
                        push_move_unchecked(moves, m, counter);
                    }
                }
            };
            process_target(target.uleft());
            process_target(target.uright());
        }
    }

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut counter = 0;
        macro_rules! push_moves {
            ($moves:expr, $piece:expr, $board:expr) => {
                for sq in $board {
                    let moveinfo = $piece.psuedo_legal_targets_fast(self, &sq);
                    for t in moveinfo.targets {
                        let m = Move::infer(sq, t, self);
                        push_move_unchecked(&mut $moves, m, &mut counter);
                    }
                }
            };
        }

        match self.turn {
            PieceColor::White => {
                let num_pawns = self.white_pawns.popcnt() as usize;
                let maximum_move_count = num_pawns * 4
                    + self.white_knights.popcnt() as usize * 8
                    + self.white_bishops.popcnt() as usize * 13
                    + self.white_rooks.popcnt() as usize * 14
                    + self.white_queens.popcnt() as usize * 27
                    + self.white_kings.popcnt() as usize * 8;
                let capacity = maximum_move_count;
                let mut moves = Vec::with_capacity(capacity);
                unsafe {
                    // Generating grouped pawn moves when there are no pawns
                    // can be slow
                    if num_pawns != 0 {
                        // TODO: fix and reenable grouped pawn move generation
                        push_moves!(moves, PieceType::Pawn, self.white_pawns);
                        // self.generate_grouped_psuedo_legal_white_pawn_moves(
                        //     &mut moves,
                        //     &mut counter,
                        // );
                    }
                    push_moves!(moves, PieceType::Knight, self.white_knights);
                    push_moves!(moves, PieceType::Bishop, self.white_bishops);
                    push_moves!(moves, PieceType::Rook, self.white_rooks);
                    push_moves!(moves, PieceType::Queen, self.white_queens);
                    push_moves!(moves, PieceType::King, self.white_kings);
                    moves.set_len(counter);
                }
                moves
            }
            PieceColor::Black => {
                let num_pawns = self.black_pawns.popcnt() as usize;
                let maximum_move_count = num_pawns * 4
                    + self.black_knights.popcnt() as usize * 8
                    + self.black_bishops.popcnt() as usize * 13
                    + self.black_rooks.popcnt() as usize * 14
                    + self.black_queens.popcnt() as usize * 27
                    + self.black_kings.popcnt() as usize * 8;
                let capacity = maximum_move_count;
                let mut moves = Vec::with_capacity(capacity);
                unsafe {
                    if num_pawns != 0 {
                        push_moves!(moves, PieceType::Pawn, self.black_pawns);
                        // self.generate_grouped_psuedo_legal_black_pawn_moves(
                        //     &mut moves,
                        //     &mut counter,
                        // );
                    }
                    push_moves!(moves, PieceType::Knight, self.black_knights);
                    push_moves!(moves, PieceType::Bishop, self.black_bishops);
                    push_moves!(moves, PieceType::Rook, self.black_rooks);
                    push_moves!(moves, PieceType::Queen, self.black_queens);
                    push_moves!(moves, PieceType::King, self.black_kings);
                    moves.set_len(counter);
                }
                moves
            }
        }
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

#[inline(always)]
unsafe fn push_move_unchecked(moves: &mut Vec<Move>, m: Move, counter: &mut usize) {
    debug_assert!(
        *counter < moves.capacity(),
        "{:?}, {:?}",
        counter,
        moves.capacity()
    );
    debug_assert_ne!(*counter, usize::MAX);
    unsafe {
        moves.as_mut_ptr().add(*counter).write(m);
        *counter = counter.unchecked_add(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::bitboard::{BitBoard, EMPTY};
    use crate::movegen::moves::Move;
    use crate::movegen::pieces::piece::{PieceColor, PieceType};
    use crate::position::game::Game;
    use crate::position::game::{STARTING_FEN, State};
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
            game.determine_color(BitBoard::from_square(white)),
            Some(PieceColor::White)
        );
        assert_eq!(game.determine_color(BitBoard::from_square(empty)), None);
        assert_eq!(
            game.determine_color(BitBoard::from_square(black)),
            Some(PieceColor::Black)
        );
        assert_eq!(
            game.determine_color(BitBoard::from_square(queen)),
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
            game.piece_lookup(pawn).map(|(p, _)| p),
            Some(PieceType::Pawn)
        );
        assert_eq!(game.piece_lookup(empty), None);
        assert_eq!(
            game.piece_lookup(knight).map(|(p, _)| p),
            Some(PieceType::Knight)
        );
        assert_eq!(
            game.piece_lookup(queen).map(|(p, _)| p),
            Some(PieceType::Queen)
        );
    }

    #[test]
    fn get_occupied_bitboards() {
        let game = Game::default();

        let white_pawns = {
            let this = &game;
            let piece: &PieceType = &PieceType::Pawn;
            let color: &PieceColor = &PieceColor::White;
            *this.get_pieces(piece, color)
        };
        assert_eq!(white_pawns, game.white_pawns);
        assert!(BitBoard::from_square(Square::A2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::H2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::A3) & white_pawns == EMPTY);
        assert!(BitBoard::from_square(Square::E4) & white_pawns == EMPTY);

        let black_rooks = {
            let this = &game;
            let piece: &PieceType = &PieceType::Rook;
            let color: &PieceColor = &PieceColor::Black;
            *this.get_pieces(piece, color)
        };
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

    #[test]
    fn generate_grouped_white_pawn_moves_equals_individual_pawn_moves() {
        let game = Game::default();
        let mut individual = Vec::new();
        for sq in game.white_pawns {
            let moveinfo = PieceType::Pawn.psuedo_legal_targets_fast(&game, &sq);
            for t in moveinfo.targets {
                let m = Move::infer(sq, t, &game);
                individual.push(m);
            }
        }

        let mut counter = 0;
        let mut grouped = Vec::with_capacity(100);
        unsafe {
            game.generate_grouped_psuedo_legal_white_pawn_moves(&mut grouped, &mut counter);
            grouped.set_len(counter);
        }

        println!(
            "\nGrouped: {}\nIndividual: {}",
            format_pretty_list(&grouped),
            format_pretty_list(&individual)
        );

        for m in &individual {
            should_generate(&grouped, m);
        }
        assert_eq!(grouped.len(), individual.len());
    }

    #[test]
    fn generate_grouped_black_pawn_moves_equals_individual_pawn_moves() {
        let mut game = Game::default();
        game.play(&Move::infer(Square::E2, Square::E4, &game));
        let mut individual = Vec::new();
        for sq in game.black_pawns {
            let moveinfo = PieceType::Pawn.psuedo_legal_targets_fast(&game, &sq);
            for t in moveinfo.targets {
                let m = Move::infer(sq, t, &game);
                individual.push(m);
            }
        }

        let mut counter = 0;
        let mut grouped = Vec::with_capacity(100);
        unsafe {
            game.generate_grouped_psuedo_legal_black_pawn_moves(&mut grouped, &mut counter);
            grouped.set_len(counter);
        }

        println!(
            "\nGrouped: {}\nIndividual: {}",
            format_pretty_list(&grouped),
            format_pretty_list(&individual)
        );

        for m in &individual {
            should_generate(&grouped, m);
        }
        assert_eq!(grouped.len(), individual.len());
    }
}
