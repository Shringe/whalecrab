mod common;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;
use whalecrab::{
    board::Board,
    movegen::pieces::{
        bishop::Bishop, king::King, knight::Knight, pawn::Pawn, piece::Piece, queen::Queen,
        rook::Rook,
    },
};

fn generate_all_psuedo_legal_pawn_moves(board: &mut Board) {
    for sq in board.white_pawns | board.black_pawns {
        Pawn(sq).psuedo_legal_moves(board);
    }
}

fn generate_all_psuedo_legal_rook_moves(board: &mut Board) {
    for sq in board.white_rooks | board.black_rooks {
        Rook(sq).psuedo_legal_moves(board);
    }
}

fn generate_all_psuedo_legal_knight_moves(board: &mut Board) {
    for sq in board.white_knights | board.black_knights {
        Knight(sq).psuedo_legal_moves(board);
    }
}

fn generate_all_psuedo_legal_bishop_moves(board: &mut Board) {
    for sq in board.white_bishops | board.black_bishops {
        Bishop(sq).psuedo_legal_moves(board);
    }
}

fn generate_all_psuedo_legal_queen_moves(board: &mut Board) {
    for sq in board.white_queens | board.black_queens {
        Queen(sq).psuedo_legal_moves(board);
    }
}

fn generate_all_psuedo_legal_king_moves(board: &mut Board) {
    for sq in board.white_kings | board.black_kings {
        King(sq).psuedo_legal_moves(board);
    }
}

fn bench(c: &mut Criterion) {
    let mut board = common::midgame_board();

    c.bench_function("Generate all legal moves", |b| {
        b.iter(|| board.generate_all_legal_moves());
    });

    c.bench_function("Generate all psuedo legal moves", |b| {
        b.iter(|| board.generate_all_psuedo_legal_moves());
    });

    c.bench_function("Generate all psuedo legal pawn moves", |b| {
        b.iter(|| generate_all_psuedo_legal_pawn_moves(&mut board))
    });

    c.bench_function("Generate all psuedo legal rook moves", |b| {
        b.iter(|| generate_all_psuedo_legal_rook_moves(&mut board))
    });

    c.bench_function("Generate all psuedo legal knight moves", |b| {
        b.iter(|| generate_all_psuedo_legal_knight_moves(&mut board))
    });

    c.bench_function("Generate all psuedo legal bishop moves", |b| {
        b.iter(|| generate_all_psuedo_legal_bishop_moves(&mut board))
    });

    c.bench_function("Generate all psuedo legal queen moves", |b| {
        b.iter(|| generate_all_psuedo_legal_queen_moves(&mut board))
    });

    c.bench_function("Generate all psuedo legal king moves", |b| {
        b.iter(|| generate_all_psuedo_legal_king_moves(&mut board))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(2000));
    targets = bench
}
criterion_main!(benches);
