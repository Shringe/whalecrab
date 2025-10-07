use crate::{
    bitboard::{BitBoard, EMPTY},
    castling::{self, CastleSide},
    game::Game,
    movegen::{
        moves::{get_targets, Move, MoveType},
        pieces::piece::{Color, PieceType},
    },
};

impl Move {
    fn unplay_normal(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&tobb)
            .expect("Couldn't find piece to unmove!");

        // Move piece back
        let pieces = game.get_pieces_mut(&piece, &color);
        *pieces ^= tobb;
        *pieces |= frombb;

        self.restore_attack_boards(game, &piece, &color);
    }

    fn unplay_capture(&self, game: &mut Game, piece_type: &PieceType) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&tobb)
            .expect("Couldn't find piece to unmove!");
        let enemy_color = color.opponent();

        // Move piece back
        let pieces = game.get_pieces_mut(&piece, &color);
        *pieces ^= tobb;
        *pieces |= frombb;

        // Restore captured enemy piece
        let enemy_pieces = game.get_pieces_mut(piece_type, &enemy_color);
        *enemy_pieces |= tobb;

        self.restore_attack_boards(game, &piece, &color);
    }

    fn unplay_create_en_passant(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&tobb)
            .expect("Couldn't find piece to unmove!");

        // Move piece back
        let pieces = game.get_pieces_mut(&piece, &color);
        *pieces ^= tobb;
        *pieces |= frombb;

        self.restore_attack_boards(game, &piece, &color);
    }

    fn unplay_capture_en_passant(&self, game: &mut Game) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let (piece, color) = game
            .determine_piece(&tobb)
            .expect("Couldn't find piece to unmove!");
        let enemy_color = color.opponent();

        // Move piece back
        let pieces = game.get_pieces_mut(&piece, &color);
        *pieces ^= tobb;
        *pieces |= frombb;

        // Restore the captured pawn
        let en_passant_bb = BitBoard::from_square(
            self.to
                .backward(&color)
                .expect("Can't find pawn in front of en_passant_target!"),
        );
        let enemy_pawns = game.get_pieces_mut(&PieceType::Pawn, &enemy_color);
        *enemy_pawns |= en_passant_bb;

        self.restore_attack_boards(game, &piece, &color);
    }

    fn unplay_promotion(&self, game: &mut Game, promoted_piece: &PieceType) {
        let frombb = BitBoard::from_square(self.from);
        let tobb = BitBoard::from_square(self.to);
        let color = game
            .determine_color(&tobb)
            .expect("Couldn't find piece to unmove!");

        // Remove promoted piece from destination square
        let promoted_pieces = game.get_pieces_mut(promoted_piece, &color);
        *promoted_pieces ^= tobb;

        // Restore original pawn
        let pawns = game.get_pieces_mut(&PieceType::Pawn, &color);
        *pawns |= frombb;

        self.restore_attack_boards(game, &PieceType::Pawn, &color);
    }

    fn unplay_castle(&self, game: &mut Game, castle_side: &CastleSide) {
        let frombb = BitBoard::from_square(self.from);
        let color = game
            .determine_color(&frombb)
            .expect("Couldn't find king to unmove!");

        match &color {
            Color::White => match castle_side {
                CastleSide::Queenside => {
                    game.position.white_kings ^= castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                    game.position.white_rooks ^= castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                }
                CastleSide::Kingside => {
                    game.position.white_kings ^= castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                    game.position.white_rooks ^= castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                }
            },

            Color::Black => match castle_side {
                CastleSide::Queenside => {
                    game.position.black_kings ^= castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                    game.position.black_rooks ^= castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                }
                CastleSide::Kingside => {
                    game.position.black_kings ^= castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                    game.position.black_rooks ^= castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                }
            },
        }
    }

    /// Helper method to restore attack boards when unmaking a move
    fn restore_attack_boards(&self, game: &mut Game, piece: &PieceType, color: &Color) {
        let enemy_color = color.opponent();

        match piece {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
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

    /// Unplays a move on the board
    /// Currently restores:
    /// - [ ] Castling rights
    /// - [ ] En passant target
    /// - [ ] Halfmove timeout
    /// - [ ] Fullmove clock
    /// - [ ] Current turn color
    pub fn unplay(&self, game: &mut Game) {
        match &self.variant {
            MoveType::Normal => self.unplay_normal(game),
            MoveType::Capture(piece_type) => self.unplay_capture(game, piece_type),
            MoveType::CreateEnPassant => self.unplay_create_en_passant(game),
            MoveType::CaptureEnPassant => self.unplay_capture_en_passant(game),
            MoveType::Promotion(piece_type) => self.unplay_promotion(game, piece_type),
            MoveType::Castle(castle_side) => self.unplay_castle(game, castle_side),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    macro_rules! play_unplay_with_game {
        ($game:expr, $sequence:expr) => {{
            let mut game = $game;
            let before = game.position.clone();
            let mut has_played = Vec::new();
            for (from, to) in $sequence {
                let m = Move::new(from, to, &game.position);
                m.play(&mut game);
                has_played.push(m);
            }

            has_played.reverse();
            for m in has_played {
                m.unplay(&mut game);
            }

            let after = game.position;
            assert_eq!(before, after);
        }};
    }

    macro_rules! play_unplay {
        ($sequence:expr) => {{
            let game = Game::default();
            play_unplay_with_game!(game, $sequence)
        }};
        ($fen:expr, $sequence:expr) => {{
            let game = Game::from_position(Board::from_fen($fen).unwrap());
            play_unplay_with_game!(game, $sequence)
        }};
    }

    macro_rules! test_play_unplay {
        ($test_name:ident, $sequence:expr) => {
            #[test]
            fn $test_name() {
                play_unplay!($sequence);
            }
        };
        ($test_name:ident, $fen:expr, $sequence:expr) => {
            #[test]
            fn $test_name() {
                play_unplay!($fen, $sequence);
            }
        };
    }

    test_play_unplay!(
        play_unplay_large_sequence,
        [
            (Square::E2, Square::E4),
            (Square::D7, Square::D5),
            (Square::E4, Square::E5),
            (Square::D5, Square::D4),
            (Square::C2, Square::C4),
            (Square::D4, Square::C3), // En passant capture
        ]
    );

    test_play_unplay!(unplay_normal, [(Square::E2, Square::E4)]);
}
