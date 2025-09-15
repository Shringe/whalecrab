use crate::{
    bitboard::{BitBoard, EMPTY},
    castling::CastlingRights,
    file::File,
    movegen::{
        moves::{get_targets, Move},
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

#[derive(Debug, PartialEq, Clone, Hash)]
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
    pub fn get_psuedo_legal_moves(&self, board: &Board, square: Square) -> Vec<Move> {
        match self {
            PieceType::Pawn => Pawn(square).psuedo_legal_moves(board),
            PieceType::Knight => Knight(square).psuedo_legal_moves(board),
            PieceType::Bishop => Bishop(square).psuedo_legal_moves(board),
            PieceType::Rook => Rook(square).psuedo_legal_moves(board),
            PieceType::Queen => Queen(square).psuedo_legal_moves(board),
            PieceType::King => King(square).psuedo_legal_moves(board),
        }
    }

    pub fn get_legal_moves(&self, board: &mut Board, square: Square) -> Vec<Move> {
        match self {
            PieceType::Pawn => Pawn(square).legal_moves(board),
            PieceType::Knight => Knight(square).legal_moves(board),
            PieceType::Bishop => Bishop(square).legal_moves(board),
            PieceType::Rook => Rook(square).legal_moves(board),
            PieceType::Queen => Queen(square).legal_moves(board),
            PieceType::King => King(square).legal_moves(board),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Board {
    pub white_pawn_bitboard: BitBoard,
    pub white_knight_bitboard: BitBoard,
    pub white_bishop_bitboard: BitBoard,
    pub white_rook_bitboard: BitBoard,
    pub white_queen_bitboard: BitBoard,
    pub white_king_bitboard: BitBoard,

    pub black_pawn_bitboard: BitBoard,
    pub black_knight_bitboard: BitBoard,
    pub black_bishop_bitboard: BitBoard,
    pub black_rook_bitboard: BitBoard,
    pub black_queen_bitboard: BitBoard,
    pub black_king_bitboard: BitBoard,

    pub en_passant_target: Option<Square>,
    pub turn: Color,

    pub castling_rights: CastlingRights,
    pub transposition_table: HashMap<u64, f32>,
    pub white_attack_bitboard: BitBoard,
    pub black_attack_bitboard: BitBoard,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            white_pawn_bitboard: EMPTY,
            white_knight_bitboard: EMPTY,
            white_bishop_bitboard: EMPTY,
            white_rook_bitboard: EMPTY,
            white_queen_bitboard: EMPTY,
            white_king_bitboard: EMPTY,

            black_pawn_bitboard: EMPTY,
            black_knight_bitboard: EMPTY,
            black_bishop_bitboard: EMPTY,
            black_rook_bitboard: EMPTY,
            black_queen_bitboard: EMPTY,
            black_king_bitboard: EMPTY,

            en_passant_target: None,
            turn: Color::White,

            castling_rights: CastlingRights::empty(),
            transposition_table: HashMap::new(),
            white_attack_bitboard: EMPTY,
            black_attack_bitboard: EMPTY,
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

        let rows = body_fen.split('/');
        let mut new = Board::empty();

        for (rank, row) in rows.rev().enumerate() {
            let mut file = 0;
            for c in row.chars() {
                let sq = BitBoard::set(Rank::from_index(rank), File::from_index(file));
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
            fen.push_str(&target.to_string());
        } else {
            fen.push('-');
        }

        // Placeholder as the Halfmove and Fullmove clock are not implemented
        fen.push_str(" 0 0");

        fen
    }

    pub fn set_occupied_bitboard(&mut self, piece: &PieceType, color: &Color, new: BitBoard) {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawn_bitboard = new,
                PieceType::Knight => self.white_knight_bitboard = new,
                PieceType::Bishop => self.white_bishop_bitboard = new,
                PieceType::Rook => self.white_rook_bitboard = new,
                PieceType::Queen => self.white_queen_bitboard = new,
                PieceType::King => self.white_king_bitboard = new,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawn_bitboard = new,
                PieceType::Knight => self.black_knight_bitboard = new,
                PieceType::Bishop => self.black_bishop_bitboard = new,
                PieceType::Rook => self.black_rook_bitboard = new,
                PieceType::Queen => self.black_queen_bitboard = new,
                PieceType::King => self.black_king_bitboard = new,
            },
        }
    }

    pub fn get_occupied_bitboard(&self, piece: &PieceType, color: &Color) -> BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.white_pawn_bitboard,
                PieceType::Knight => self.white_knight_bitboard,
                PieceType::Bishop => self.white_bishop_bitboard,
                PieceType::Rook => self.white_rook_bitboard,
                PieceType::Queen => self.white_queen_bitboard,
                PieceType::King => self.white_king_bitboard,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.black_pawn_bitboard,
                PieceType::Knight => self.black_knight_bitboard,
                PieceType::Bishop => self.black_bishop_bitboard,
                PieceType::Rook => self.black_rook_bitboard,
                PieceType::Queen => self.black_queen_bitboard,
                PieceType::King => self.black_king_bitboard,
            },
        }
    }

    pub fn occupied_white_bitboard(&self) -> BitBoard {
        self.white_pawn_bitboard
            | self.white_knight_bitboard
            | self.white_bishop_bitboard
            | self.white_rook_bitboard
            | self.white_queen_bitboard
            | self.white_king_bitboard
    }

    pub fn occupied_black_bitboard(&self) -> BitBoard {
        self.black_pawn_bitboard
            | self.black_knight_bitboard
            | self.black_bishop_bitboard
            | self.black_rook_bitboard
            | self.black_queen_bitboard
            | self.black_king_bitboard
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

        if pos & (self.white_pawn_bitboard | self.black_pawn_bitboard) != EMPTY {
            Some(PieceType::Pawn)
        } else if pos & (self.white_knight_bitboard | self.black_knight_bitboard) != EMPTY {
            Some(PieceType::Knight)
        } else if pos & (self.white_bishop_bitboard | self.black_bishop_bitboard) != EMPTY {
            Some(PieceType::Bishop)
        } else if pos & (self.white_rook_bitboard | self.black_rook_bitboard) != EMPTY {
            Some(PieceType::Rook)
        } else if pos & (self.white_queen_bitboard | self.black_queen_bitboard) != EMPTY {
            Some(PieceType::Queen)
        } else if pos & (self.white_king_bitboard | self.black_king_bitboard) != EMPTY {
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

    /// Determines whether the opponent's king is in check
    pub fn is_king_in_check(&self) -> bool {
        let targets =
            BitBoard::from_square_vec(get_targets(self.generate_all_psuedo_legal_moves()));
        let king = self.get_occupied_bitboard(&PieceType::King, &self.turn.opponent());
        king & targets != EMPTY
    }

    /// Generates all psuedo legal moves for the current player
    pub fn generate_all_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = match self.turn {
            Color::White => self.occupied_white_bitboard(),
            Color::Black => self.occupied_black_bitboard(),
        };

        for sq in occupied {
            if let Some(piece) = self.determine_piece(sq) {
                moves.extend(piece.get_psuedo_legal_moves(self, sq))
            }
        }

        moves
    }

    /// Generates all legal moves for the current player
    pub fn generate_all_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = match self.turn {
            Color::White => self.occupied_white_bitboard(),
            Color::Black => self.occupied_black_bitboard(),
        };

        for sq in occupied {
            if let Some(piece) = self.determine_piece(sq) {
                moves.extend(piece.get_legal_moves(self, sq))
            }
        }

        moves
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            white_pawn_bitboard: BitBoard::INITIAL_WHITE_PAWN,
            white_knight_bitboard: BitBoard::INITIAL_WHITE_KNIGHT,
            white_bishop_bitboard: BitBoard::INITIAL_WHITE_BISHOP,
            white_rook_bitboard: BitBoard::INITIAL_WHITE_ROOK,
            white_queen_bitboard: BitBoard::INITIAL_WHITE_QUEEN,
            white_king_bitboard: BitBoard::INITIAL_WHITE_KING,

            black_pawn_bitboard: BitBoard::INITIAL_BLACK_PAWN,
            black_knight_bitboard: BitBoard::INITIAL_BLACK_KNIGHT,
            black_bishop_bitboard: BitBoard::INITIAL_BLACK_BISHOP,
            black_rook_bitboard: BitBoard::INITIAL_BLACK_ROOK,
            black_queen_bitboard: BitBoard::INITIAL_BLACK_QUEEN,
            black_king_bitboard: BitBoard::INITIAL_BLACK_KING,

            en_passant_target: None,
            turn: Color::White,

            castling_rights: CastlingRights::default(),
            transposition_table: HashMap::default(),
            white_attack_bitboard: EMPTY,
            black_attack_bitboard: EMPTY,
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board(\"{}\")", self.to_fen())
    }
}

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.white_pawn_bitboard.hash(state);
        self.white_knight_bitboard.hash(state);
        self.white_bishop_bitboard.hash(state);
        self.white_rook_bitboard.hash(state);
        self.white_queen_bitboard.hash(state);
        self.white_king_bitboard.hash(state);
        self.black_pawn_bitboard.hash(state);
        self.black_knight_bitboard.hash(state);
        self.black_bishop_bitboard.hash(state);
        self.black_rook_bitboard.hash(state);
        self.black_queen_bitboard.hash(state);
        self.black_king_bitboard.hash(state);
        self.en_passant_target.hash(state);
        self.turn.hash(state);
        self.castling_rights.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::compare_to_fen;

    #[test]
    fn white_king_in_check() {
        let white_in_check = "rnbqk1nr/pppp1ppp/8/4p3/1b1PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 1";
        let board = Board::from_fen(white_in_check).unwrap();
        assert!(board.is_king_in_check())
    }

    #[test]
    fn black_king_in_check() {
        let black_in_check = "rnbqkb1r/ppp2ppp/5n2/1B1pp3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1";
        let board = Board::from_fen(black_in_check).unwrap();
        assert!(board.is_king_in_check())
    }

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
        let mut board = Board::default();

        for m in [
            Move::new(Square::E2, Square::E4, &board),
            Move::new(Square::D7, Square::D5, &board),
            Move::new(Square::E4, Square::E5, &board),
            Move::new(Square::F7, Square::F5, &board),
        ] {
            board = m.make(&board);
        }

        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
        compare_to_fen(&board, fen);
    }

    #[test]
    fn starting_fen() {
        let board = Board::default();
        compare_to_fen(&board, STARTING_FEN);
    }

    #[test]
    fn complex_fen() {
        let mut board = Board::default();

        for m in [
            Move::new(Square::E2, Square::E4, &board),
            Move::new(Square::D7, Square::D5, &board),
            Move::new(Square::E4, Square::D5, &board),
            Move::new(Square::B8, Square::C6, &board),
            Move::new(Square::F1, Square::B5, &board),
        ] {
            board = m.make(&board);
        }

        let fen = "r1bqkbnr/ppp1pppp/2n5/1B1P4/8/8/PPPP1PPP/RNBQK1NR b KQkq - 2 3";
        compare_to_fen(&board, fen);
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
        assert_eq!(white_pawns, board.white_pawn_bitboard);
        assert!(BitBoard::from_square(Square::A2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::H2) & white_pawns != EMPTY);
        assert!(BitBoard::from_square(Square::A3) & white_pawns == EMPTY);
        assert!(BitBoard::from_square(Square::E4) & white_pawns == EMPTY);

        let black_rooks = board.get_occupied_bitboard(&PieceType::Rook, &Color::Black);
        assert_eq!(black_rooks, board.black_rook_bitboard);
        assert!(BitBoard::from_square(Square::A8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::H8) & black_rooks != EMPTY);
        assert!(BitBoard::from_square(Square::B7) & black_rooks == EMPTY);
        assert!(BitBoard::from_square(Square::E5) & black_rooks == EMPTY);
    }
}
