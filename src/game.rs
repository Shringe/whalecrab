use std::collections::HashMap;

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{color_field_getters, Board, Color, PieceType},
};

pub struct Game {
    pub position: Board,
    pub transposition_table: HashMap<u64, f32>,
    pub white_num_checks: u8,
    pub black_num_checks: u8,
    pub white_attacks: BitBoard,
    pub black_attacks: BitBoard,
    pub white_check_rays: BitBoard,
    pub black_check_rays: BitBoard,
    pub white_occupied: BitBoard,
    pub black_occupied: BitBoard,
    pub occupied: BitBoard,
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Self {
            position: Board::default(),
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
        };

        game.refresh();
        game
    }
}

impl Game {
    color_field_getters!(attacks, BitBoard);
    color_field_getters!(check_rays, BitBoard);
    color_field_getters!(num_checks, u8);

    /// Recalculates certain cached values regarding the position
    /// Should be called on Self initialization and position updates
    fn refresh(&mut self) {
        let white_occupied = self.position.white_pawns
            | self.position.white_knights
            | self.position.white_bishops
            | self.position.white_rooks
            | self.position.white_queens
            | self.position.white_kings;
        let black_occupied = self.position.black_pawns
            | self.position.black_knights
            | self.position.black_bishops
            | self.position.black_rooks
            | self.position.black_queens
            | self.position.black_kings;
        let occupied = white_occupied | black_occupied;

        self.white_occupied = white_occupied;
        self.black_occupied = black_occupied;
        self.occupied = occupied;
    }

    pub fn get_occupied_bitboard(&self, piece: &PieceType, color: &Color) -> BitBoard {
        match color {
            Color::White => match piece {
                PieceType::Pawn => self.position.white_pawns,
                PieceType::Knight => self.position.white_knights,
                PieceType::Bishop => self.position.white_bishops,
                PieceType::Rook => self.position.white_rooks,
                PieceType::Queen => self.position.white_queens,
                PieceType::King => self.position.white_kings,
            },
            Color::Black => match piece {
                PieceType::Pawn => self.position.black_pawns,
                PieceType::Knight => self.position.black_knights,
                PieceType::Bishop => self.position.black_bishops,
                PieceType::Rook => self.position.black_rooks,
                PieceType::Queen => self.position.black_queens,
                PieceType::King => self.position.black_kings,
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

    // /// Generates all psuedo legal moves for the current player
    // pub fn generate_all_psuedo_legal_moves(&mut self) -> Vec<Move> {
    //     let mut moves = Vec::new();
    //     let occupied = match self.turn {
    //         Color::White => self.occupied_white_bitboard(),
    //         Color::Black => self.occupied_black_bitboard(),
    //     };
    //
    //     for sq in occupied {
    //         if let Some(piece) = self.determine_piece(sq) {
    //             moves.extend(piece.get_psuedo_legal_moves(self, sq))
    //         }
    //     }
    //
    //     moves
    // }
    //
    // /// Generates all legal moves for the current player
    // pub fn generate_all_legal_moves(&mut self) -> Vec<Move> {
    //     let mut moves = Vec::new();
    //     let occupied = match self.turn {
    //         Color::White => self.occupied_white_bitboard(),
    //         Color::Black => self.occupied_black_bitboard(),
    //     };
    //
    //     for sq in occupied {
    //         if let Some(piece) = self.determine_piece(sq) {
    //             moves.extend(piece.get_legal_moves(self, sq))
    //         }
    //     }
    //
    //     moves
    // }
}
