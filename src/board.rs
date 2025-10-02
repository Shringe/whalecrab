use crate::{
    bitboard::{BitBoard, EMPTY},
    castling::CastlingRights,
    file::File,
    game::Game,
    movegen::{
        moves::Move,
        pieces::{
            bishop::Bishop, king::King, knight::Knight, pawn::Pawn, piece::Piece, queen::Queen,
            rook::Rook,
        },
    },
    rank::Rank,
    square::Square,
};
use std::{collections::HashMap, fmt, hash::Hash, str::FromStr};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

impl PieceType {
    pub fn get_psuedo_legal_moves(&self, game: &mut Game, square: Square) -> Vec<Move> {
        match self {
            PieceType::Pawn => Pawn(square).psuedo_legal_moves(game),
            PieceType::Knight => Knight(square).psuedo_legal_moves(game),
            PieceType::Bishop => Bishop(square).psuedo_legal_moves(game),
            PieceType::Rook => Rook(square).psuedo_legal_moves(game),
            PieceType::Queen => Queen(square).psuedo_legal_moves(game),
            PieceType::King => King(square).psuedo_legal_moves(game),
        }
    }

    pub fn get_legal_moves(&self, game: &mut Game, square: Square) -> Vec<Move> {
        match self {
            PieceType::Pawn => Pawn(square).legal_moves(game),
            PieceType::Knight => Knight(square).legal_moves(game),
            PieceType::Bishop => Bishop(square).legal_moves(game),
            PieceType::Rook => Rook(square).legal_moves(game),
            PieceType::Queen => Queen(square).legal_moves(game),
            PieceType::King => King(square).legal_moves(game),
        }
    }

    pub fn is_ray_piece(&self) -> bool {
        match self {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => true,
            _ => false,
        }
    }
}

macro_rules! color_field_getters {
    ($field_name:ident, $return_type:ty) => {
        paste::paste! {
            pub fn [<get_ $field_name _mut>](&mut self, color: &crate::board::Color) -> &mut $return_type {
                match color {
                    crate::board::Color::White => &mut self.[<white_ $field_name>],
                    crate::board::Color::Black => &mut self.[<black_ $field_name>],
                }
            }

            pub fn [<get_ $field_name>](&self, color: &crate::board::Color) -> &$return_type {
                match color {
                    crate::board::Color::White => &self.[<white_ $field_name>],
                    crate::board::Color::Black => &self.[<black_ $field_name>],
                }
            }
        }
    };
}
pub(crate) use color_field_getters;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    InProgress,
    Checkmate,
    Stalemate,
    Timeout,
    Repetition,
}

#[derive(Clone)]
pub struct Board {
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
    pub turn: Color,

    pub half_move_timeout: usize,
    pub full_move_clock: usize,
    pub state: State,
    pub seen_positions: HashMap<u64, u8>,
    pub hash: u64,
}

impl Board {
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
            turn: Color::White,

            half_move_timeout: 0,
            full_move_clock: 0,
            state: State::InProgress,
            seen_positions: HashMap::new(),
            hash: 0,
        }
    }

    /// Takes a fen string, parses and converts it into a board position.
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
        let mut new = Board::empty();

        for (rank, row) in rows.rev().enumerate() {
            let mut file = 0;
            for c in row.chars() {
                let sq = BitBoard::from_rank_file(Rank::from_index(rank), File::from_index(file));
                let colored_piece = match c {
                    'p' => Some((PieceType::Pawn, Color::Black)),
                    'n' => Some((PieceType::Knight, Color::Black)),
                    'b' => Some((PieceType::Bishop, Color::Black)),
                    'r' => Some((PieceType::Rook, Color::Black)),
                    'q' => Some((PieceType::Queen, Color::Black)),
                    'k' => Some((PieceType::King, Color::Black)),
                    'P' => Some((PieceType::Pawn, Color::White)),
                    'N' => Some((PieceType::Knight, Color::White)),
                    'B' => Some((PieceType::Bishop, Color::White)),
                    'R' => Some((PieceType::Rook, Color::White)),
                    'Q' => Some((PieceType::Queen, Color::White)),
                    'K' => Some((PieceType::King, Color::White)),
                    _ => None,
                };

                if let Some((piece, color)) = colored_piece {
                    new.set_occupied_bitboard(
                        &piece,
                        &color,
                        new.get_occupied_bitboard(&piece, &color) | sq,
                    );

                    file += 1;
                } else {
                    file += c.to_digit(10)? as usize;
                }
            }
        }

        new.turn = if turn_fen == "b" {
            Color::Black
        } else {
            Color::White
        };

        if castling_fen != "-" {
            new.castling_rights = CastlingRights {
                white_queenside: castling_fen.contains('Q'),
                white_kingside: castling_fen.contains('K'),
                black_queenside: castling_fen.contains('q'),
                black_kingside: castling_fen.contains('k'),
            }
        }

        if let Ok(sq) = Square::from_str(en_passant_fen) {
            new.en_passant_target = Some(sq);
        }

        if let Ok(half_moves) = half_move_fen.parse() {
            new.half_move_timeout = half_moves;
        }

        if let Ok(half_moves) = full_move_fen.parse() {
            new.full_move_clock = half_moves;
        }

        Some(new)
    }

    /// Attempts to generate a fen from the current board state
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count: u8 = 0;

            for file in 0..8 {
                let square = Square::make_square(Rank::from_index(rank), File::from_index(file));

                if let Some(piece) = self.determine_piece(square) {
                    // If we had empty squares, add the count first
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    let piece_char = match self.determine_color(square).unwrap() {
                        Color::White => match piece {
                            PieceType::Pawn => 'P',
                            PieceType::Knight => 'N',
                            PieceType::Bishop => 'B',
                            PieceType::Rook => 'R',
                            PieceType::Queen => 'Q',
                            PieceType::King => 'K',
                        },

                        Color::Black => match piece {
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
            Color::White => 'w',
            Color::Black => 'b',
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

    pub fn set_occupied_bitboard(&mut self, piece: &PieceType, color: &Color, new: BitBoard) {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawns = new,
                PieceType::Knight => self.white_knights = new,
                PieceType::Bishop => self.white_bishops = new,
                PieceType::Rook => self.white_rooks = new,
                PieceType::Queen => self.white_queens = new,
                PieceType::King => self.white_kings = new,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawns = new,
                PieceType::Knight => self.black_knights = new,
                PieceType::Bishop => self.black_bishops = new,
                PieceType::Rook => self.black_rooks = new,
                PieceType::Queen => self.black_queens = new,
                PieceType::King => self.black_kings = new,
            },
        }
    }

    pub fn get_occupied_bitboard(&self, piece: &PieceType, color: &Color) -> BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawns,
                PieceType::Knight => self.white_knights,
                PieceType::Bishop => self.white_bishops,
                PieceType::Rook => self.white_rooks,
                PieceType::Queen => self.white_queens,
                PieceType::King => self.white_kings,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawns,
                PieceType::Knight => self.black_knights,
                PieceType::Bishop => self.black_bishops,
                PieceType::Rook => self.black_rooks,
                PieceType::Queen => self.black_queens,
                PieceType::King => self.black_kings,
            },
        }
    }

    pub fn occupied_white_bitboard(&self) -> BitBoard {
        self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_kings
    }

    pub fn occupied_black_bitboard(&self) -> BitBoard {
        self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_kings
    }

    pub fn occupied_bitboard(&self) -> BitBoard {
        self.occupied_white_bitboard() | self.occupied_black_bitboard()
    }

    /// Determines color of standing piece
    pub fn determine_color(&self, sq: Square) -> Option<Color> {
        let pos = BitBoard::from_square(sq);
        if pos & self.occupied_white_bitboard() != EMPTY {
            Some(Color::White)
        } else if pos & self.occupied_black_bitboard() != EMPTY {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Determines type of standing piece
    pub fn determine_piece(&self, sq: Square) -> Option<PieceType> {
        let pos = BitBoard::from_square(sq);
        if pos & self.occupied_bitboard() == EMPTY {
            return None;
        }

        if pos & (self.white_pawns | self.black_pawns) != EMPTY {
            Some(PieceType::Pawn)
        } else if pos & (self.white_knights | self.black_knights) != EMPTY {
            Some(PieceType::Knight)
        } else if pos & (self.white_bishops | self.black_bishops) != EMPTY {
            Some(PieceType::Bishop)
        } else if pos & (self.white_rooks | self.black_rooks) != EMPTY {
            Some(PieceType::Rook)
        } else if pos & (self.white_queens | self.black_queens) != EMPTY {
            Some(PieceType::Queen)
        } else if pos & (self.white_kings | self.black_kings) != EMPTY {
            Some(PieceType::King)
        } else {
            panic!("Can't determine piece type of square {:?}!", sq);
        }
    }

    /// Switches the players' turn and removes en_passant_target
    pub fn next_turn(&mut self) {
        self.turn = self.turn.opponent();
        self.en_passant_target = None;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
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
            turn: Color::White,

            half_move_timeout: 0,
            full_move_clock: 0,
            state: State::InProgress,
            seen_positions: HashMap::new(),
            hash: 0,
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board(\"{}\")", self.to_fen())
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.white_pawns == other.white_pawns
            && self.white_knights == other.white_knights
            && self.white_bishops == other.white_bishops
            && self.white_rooks == other.white_rooks
            && self.white_queens == other.white_queens
            && self.white_kings == other.white_kings
            && self.black_pawns == other.black_pawns
            && self.black_knights == other.black_knights
            && self.black_bishops == other.black_bishops
            && self.black_rooks == other.black_rooks
            && self.black_queens == other.black_queens
            && self.black_kings == other.black_kings
            && self.en_passant_target == other.en_passant_target
            && self.turn == other.turn
            && self.castling_rights == other.castling_rights
        // && self.transposition_table == other.transposition_table
        // && self.white_attack_bitboard == other.white_attack_bitboard
        // && self.black_attack_bitboard == other.black_attack_bitboard
    }
}

impl Hash for Board {
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
        self.en_passant_target.hash(state);
        self.turn.hash(state);
        self.castling_rights.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game::Game, test_utils::compare_to_fen};

    #[test]
    fn to_fen() {
        let fen_before = "rnbq1rk1/p1p2p1p/3bpp2/1p6/2pP4/2N1B3/PP1Q1PPP/R3KBNR w KQ - 4 9";
        let board_before = Board::from_fen(fen_before).unwrap();

        let fen_after = board_before.to_fen();
        let board_after = Board::from_fen(&fen_after).unwrap();

        // I'm comparing the two boards directly instead of their fen because some parts of the fen implemenation
        // are not yet implemented, which would fail the test.
        assert_eq!(board_before, board_after);
    }

    #[test]
    fn en_passant_fen() {
        let mut game = Game::default();

        for m in [
            Move::new(Square::E2, Square::E4, &game.position),
            Move::new(Square::D7, Square::D5, &game.position),
            Move::new(Square::E4, Square::E5, &game.position),
            Move::new(Square::F7, Square::F5, &game.position),
        ] {
            game.play(&m);
        }

        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
        compare_to_fen(&game.position, fen);
    }

    #[test]
    fn starting_fen() {
        let board = Board::default();
        compare_to_fen(&board, STARTING_FEN);
    }

    #[test]
    fn complex_fen() {
        let mut game = Game::default();

        for m in [
            Move::new(Square::E2, Square::E4, &game.position),
            Move::new(Square::D7, Square::D5, &game.position),
            Move::new(Square::E4, Square::D5, &game.position),
            Move::new(Square::B8, Square::C6, &game.position),
            Move::new(Square::F1, Square::B5, &game.position),
        ] {
            game.play(&m);
        }

        let fen = "r1bqkbnr/ppp1pppp/2n5/1B1P4/8/8/PPPP1PPP/RNBQK1NR b KQkq - 2 3";
        compare_to_fen(&game.position, fen);
    }

    #[test]
    fn determine_colors() {
        let board = Board::default();

        let white = Square::C2;
        let empty = Square::G4;
        let black = Square::B8;
        let queen = Square::D1;

        assert_eq!(board.determine_color(white), Some(Color::White));
        assert_eq!(board.determine_color(empty), None);
        assert_eq!(board.determine_color(black), Some(Color::Black));
        assert_eq!(board.determine_color(queen), Some(Color::White));
    }

    #[test]
    fn determine_pieces() {
        let board = Board::default();

        let pawn = Square::C2;
        let empty = Square::G4;
        let knight = Square::B8;
        let queen = Square::D8;

        assert_eq!(board.determine_piece(pawn), Some(PieceType::Pawn));
        assert_eq!(board.determine_piece(empty), None);
        assert_eq!(board.determine_piece(knight), Some(PieceType::Knight));
        assert_eq!(board.determine_piece(queen), Some(PieceType::Queen));
    }

    #[test]
    fn get_occupied_bitboards() {
        let board = Board::default();

        let white_pawns = board.get_occupied_bitboard(&PieceType::Pawn, &Color::White);
        assert_eq!(white_pawns, board.white_pawns);
        assert!(BitBoard::from_square(Square::A2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::H2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::A3) & white_pawns == EMPTY);
        assert!(BitBoard::from_square(Square::E4) & white_pawns == EMPTY);

        let black_rooks = board.get_occupied_bitboard(&PieceType::Rook, &Color::Black);
        assert_eq!(black_rooks, board.black_rooks);
        assert!(BitBoard::from_square(Square::A8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::H8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::B7) & black_rooks == EMPTY);
        assert!(BitBoard::from_square(Square::E5) & black_rooks == EMPTY);
    }
}
