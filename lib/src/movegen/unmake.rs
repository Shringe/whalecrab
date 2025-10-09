use crate::{
    bitboard::{BitBoard, EMPTY},
    castling::{self, CastleSide},
    game::Game,
    movegen::{
        moves::{get_targets, Move, MoveType},
        pieces::piece::{Color, PieceType},
    },
    square::Square,
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
    }

    // TODO, either seperate promotion and captures, or somehow restore a potential captured piece on promotion
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
    }

    fn unplay_castle(&self, game: &mut Game, castle_side: &CastleSide) {
        let to = BitBoard::from_square(self.to);
        let color = game
            .determine_color(&to)
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

    /// Unplays a move on the board.
    /// Bugs are still present.
    /// Some stuff still needs to be restored.
    /// What we will need a lookup table for:
    /// - [ ] Castling rights
    /// - [ ] Halfmove timeout
    ///
    /// What should be possible to restore:
    /// - [x] En passant target
    /// - [x] Fullmove clock
    /// - [x] Turn color
    pub fn unplay(&self, game: &mut Game) {
        match &self.variant {
            MoveType::Normal => self.unplay_normal(game),
            MoveType::Capture(piece_type) => self.unplay_capture(game, piece_type),
            MoveType::CreateEnPassant => self.unplay_create_en_passant(game),
            MoveType::CaptureEnPassant => self.unplay_capture_en_passant(game),
            MoveType::Promotion(piece_type) => self.unplay_promotion(game, piece_type),
            MoveType::Castle(castle_side) => self.unplay_castle(game, castle_side),
        }

        game.restore_position();
        game.previous_turn(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::square::Square;

    macro_rules! play_unplay_with_game {
        ($game:expr, $sequence:expr) => {{
            let mut game = $game;
            let before = game.position.clone();
            let mut has_played = Vec::new();

            println!("=== Starting play_unplay_with_game test ===");
            println!("Initial position: {:?}", before);

            for (i, (from, to)) in $sequence.iter().enumerate() {
                let m = Move::new(*from, *to, &game.position);
                println!("\n[Play {}] Move: {} -> {}", i, from, to);
                println!("  Before play: {:?}", game.position);
                m.play(&mut game);
                println!("  After play: {:?}", game.position);
                has_played.push(m);
            }

            println!("\n=== Beginning unplay sequence ===");
            has_played.reverse();
            for (i, m) in has_played.iter().enumerate() {
                println!("\n[Unplay {}] Move: {:?}", i, m);
                println!("  Before unplay: {:?}", game.position);
                m.unplay(&mut game);
                println!("  After unplay: {:?}", game.position);
            }

            let after = game.position;
            println!("\n=== Final comparison ===");
            println!("Before:  {:?}", before);
            println!("After:   {:?}", after);
            println!("Match: {}", before == after);

            assert_eq!(
                before, after,
                "\nPosition mismatch!\nExpected: {:?}\nGot: {:?}",
                before, after
            );
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

    test_play_unplay!(unplay_normal, [(Square::G1, Square::F3)]);
    test_play_unplay!(unplay_create_en_passant, [(Square::E2, Square::E4)]);

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

    test_play_unplay!(
        unplay_capture,
        "rnbqkb1r/pppppp2/8/8/8/8/PPPPP3/RNBQK2R b KQkq - 0 1",
        [(Square::H8, Square::H1)]
    );

    test_play_unplay!(
        unplay_capture_en_passant,
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 2",
        [(Square::E5, Square::F6)]
    );

    test_play_unplay!(
        unplay_promotion,
        "8/8/8/8/8/8/5Kpk/8 b - - 0 1",
        [(Square::G2, Square::G1)]
    );

    test_play_unplay!(
        unplay_castle,
        "rnbqkbnr/pppppppp/8/8/2BPPB2/P1N2N1P/1PPQ1PP1/R3K2R w KQkq - 0 1",
        [(Square::E1, Square::C1)]
    );
}
