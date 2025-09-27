use std::collections::HashMap;

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{color_field_getters, Board, Color, PieceType},
    castling::{self, CastleSide},
    movegen::moves::{get_targets, Move, MoveType},
    square::Square,
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

    /// Finishes a turn
    fn next_turn(&mut self) {
        self.position.turn = self.position.turn.opponent();
        self.position.en_passant_target = None;
        self.refresh();
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
    pub fn make(&mut self, m: &Move) {
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
                                castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                            self.position.black_rooks ^=
                                castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                        }
                        CastleSide::Kingside => {
                            self.position.black_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                            self.position.black_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                        }
                    }
                }
            }

            self.next_turn();
            return;
        }

        // Update attack bitboards
        match piece {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
                // HACK: Clone so that attack boards are not automatically updated for now
                // TODO: Implement way to movegen withhout setting attack boards
                let attack_board = *self.get_attacks(&color);
                let check_ray_board = *self.get_check_rays(&color);
                let moves = piece.get_psuedo_legal_moves(&mut self.position, m.from);
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

        // Set en passant rules and switch turn
        self.next_turn();
        if m.variant == MoveType::CreateEnPassant {
            self.position.en_passant_target = m.to.backward(&color);
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
