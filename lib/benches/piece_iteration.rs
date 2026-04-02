mod common;
use std::hint::black_box;

use criterion::Criterion;
use whalecrab_lib::{
    bitboard::BitBoard,
    game::Game,
    movegen::pieces::piece::{PieceColor, PieceType},
    square::Square,
};

fn piece_iterator(game: &Game) -> impl Iterator<Item = (Square, PieceType, PieceColor)> {
    game.white_pawns
        .map(|sq| (sq, PieceType::Pawn, PieceColor::White))
        .chain(
            game.white_knights
                .map(|sq| (sq, PieceType::Knight, PieceColor::White)),
        )
        .chain(
            game.white_bishops
                .map(|sq| (sq, PieceType::Bishop, PieceColor::White)),
        )
        .chain(
            game.white_rooks
                .map(|sq| (sq, PieceType::Rook, PieceColor::White)),
        )
        .chain(
            game.white_queens
                .map(|sq| (sq, PieceType::Queen, PieceColor::White)),
        )
        .chain(
            game.white_kings
                .map(|sq| (sq, PieceType::King, PieceColor::White)),
        )
        .chain(
            game.black_pawns
                .map(|sq| (sq, PieceType::Pawn, PieceColor::Black)),
        )
        .chain(
            game.black_knights
                .map(|sq| (sq, PieceType::Knight, PieceColor::Black)),
        )
        .chain(
            game.black_bishops
                .map(|sq| (sq, PieceType::Bishop, PieceColor::Black)),
        )
        .chain(
            game.black_rooks
                .map(|sq| (sq, PieceType::Rook, PieceColor::Black)),
        )
        .chain(
            game.black_queens
                .map(|sq| (sq, PieceType::Queen, PieceColor::Black)),
        )
        .chain(
            game.black_kings
                .map(|sq| (sq, PieceType::King, PieceColor::Black)),
        )
}

fn bench(c: &mut Criterion) {
    let game = Game::default();

    c.bench_function("Iterate using game.iterate_pieces", |b| {
        b.iter(|| {
            for (sq, piece, color) in piece_iterator(&game) {
                black_box((sq, piece, color));
            }
        });
    });

    c.bench_function("Iterate using game.occupied and piece_lookup", |b| {
        b.iter(|| {
            for sq in game.occupied {
                let (piece, color) = game.piece_lookup(sq).unwrap();
                black_box((sq, piece, color));
            }
        });
    });

    c.bench_function("Single square game.determine_color", |b| {
        let sq = Square::E8;
        let sqbb = BitBoard::from_square(sq);
        b.iter(|| {
            let color = game.determine_color(sqbb).unwrap();
            black_box(color);
        });
    });

    c.bench_function("Single square sq -> sqbb -> game.determine_color", |b| {
        let sq = Square::E8;
        b.iter(|| {
            let sqbb = BitBoard::from_square(sq);
            let color = game.determine_color(sqbb).unwrap();
            black_box(color);
        });
    });

    c.bench_function("Single square game.piece_lookup", |b| {
        let sq = Square::E1;
        b.iter(|| {
            let (piece, color) = game.piece_lookup(sq).unwrap();
            black_box((piece, color));
        });
    });

    c.bench_function("Single square sqbb -> sq -> game.piece_lookup", |b| {
        let sqbb = BitBoard::from_square(Square::E1);
        b.iter(|| {
            let sq = sqbb.to_square();
            let (piece, color) = game.piece_lookup(sq).unwrap();
            black_box((piece, color));
        });
    });
}

setup_criterion!();
