mod common;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;
use whalecrab::{
    game::Game,
    movegen::pieces::{
        bishop::Bishop, king::King, knight::Knight, pawn::Pawn, piece::Piece, queen::Queen,
        rook::Rook,
    },
};

fn generate_all_psuedo_legal_pawn_moves(game: &mut Game) {
    for sq in game.position.white_pawns | game.position.black_pawns {
        Pawn(sq).psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_rook_moves(game: &mut Game) {
    for sq in game.position.white_rooks | game.position.black_rooks {
        Rook(sq).psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_knight_moves(game: &mut Game) {
    for sq in game.position.white_knights | game.position.black_knights {
        Knight(sq).psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_bishop_moves(game: &mut Game) {
    for sq in game.position.white_bishops | game.position.black_bishops {
        Bishop(sq).psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_queen_moves(game: &mut Game) {
    for sq in game.position.white_queens | game.position.black_queens {
        Queen(sq).psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_king_moves(game: &mut Game) {
    for sq in game.position.white_kings | game.position.black_kings {
        King(sq).psuedo_legal_moves(game);
    }
}

fn bench(c: &mut Criterion) {
    let mut game = common::midgame();

    c.bench_function("Generate all legal moves", |b| {
        b.iter(|| game.generate_all_legal_moves());
    });

    c.bench_function("Generate all psuedo legal moves", |b| {
        b.iter(|| game.generate_all_psuedo_legal_moves());
    });

    c.bench_function("Generate all psuedo legal pawn moves", |b| {
        b.iter(|| generate_all_psuedo_legal_pawn_moves(&mut game))
    });

    c.bench_function("Generate all psuedo legal rook moves", |b| {
        b.iter(|| generate_all_psuedo_legal_rook_moves(&mut game))
    });

    c.bench_function("Generate all psuedo legal knight moves", |b| {
        b.iter(|| generate_all_psuedo_legal_knight_moves(&mut game))
    });

    c.bench_function("Generate all psuedo legal bishop moves", |b| {
        b.iter(|| generate_all_psuedo_legal_bishop_moves(&mut game))
    });

    c.bench_function("Generate all psuedo legal queen moves", |b| {
        b.iter(|| generate_all_psuedo_legal_queen_moves(&mut game))
    });

    c.bench_function("Generate all psuedo legal king moves", |b| {
        b.iter(|| generate_all_psuedo_legal_king_moves(&mut game))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(2000));
    targets = bench
}
criterion_main!(benches);
