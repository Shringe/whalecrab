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

fn generate_all_psuedo_legal_pawn_moves(board: &Board) {
    for sq in board.white_pawn_bitboard | board.black_pawn_bitboard {
        Pawn(sq).psuedo_legal_moves(&board);
    }
}

fn generate_all_psuedo_legal_rook_moves(board: &Board) {
    for sq in board.white_rook_bitboard | board.black_rook_bitboard {
        Rook(sq).psuedo_legal_moves(&board);
    }
}

fn generate_all_psuedo_legal_knight_moves(board: &Board) {
    for sq in board.white_knight_bitboard | board.black_knight_bitboard {
        Knight(sq).psuedo_legal_moves(&board);
    }
}

fn generate_all_psuedo_legal_bishop_moves(board: &Board) {
    for sq in board.white_bishop_bitboard | board.black_bishop_bitboard {
        Bishop(sq).psuedo_legal_moves(&board);
    }
}

fn generate_all_psuedo_legal_queen_moves(board: &Board) {
    for sq in board.white_queen_bitboard | board.black_queen_bitboard {
        Queen(sq).psuedo_legal_moves(&board);
    }
}

fn generate_all_psuedo_legal_king_moves(board: &Board) {
    for sq in board.white_king_bitboard | board.black_king_bitboard {
        King(sq).psuedo_legal_moves(&board);
    }
}

fn bench(c: &mut Criterion) {
    let board = common::midgame_board();

    c.bench_function("Generate all legal moves", |b| {
        b.iter(|| board.generate_all_legal_moves());
    });

    c.bench_function("Generate all psuedo legal moves", |b| {
        b.iter(|| board.generate_all_psuedo_legal_moves());
    });

    c.bench_function("Generate all psuedo legal pawn moves", |b| {
        b.iter(|| generate_all_psuedo_legal_pawn_moves(&board))
    });

    c.bench_function("Generate all psuedo legal rook moves", |b| {
        b.iter(|| generate_all_psuedo_legal_rook_moves(&board))
    });

    c.bench_function("Generate all psuedo legal knight moves", |b| {
        b.iter(|| generate_all_psuedo_legal_knight_moves(&board))
    });

    c.bench_function("Generate all psuedo legal bishop moves", |b| {
        b.iter(|| generate_all_psuedo_legal_bishop_moves(&board))
    });

    c.bench_function("Generate all psuedo legal queen moves", |b| {
        b.iter(|| generate_all_psuedo_legal_queen_moves(&board))
    });

    c.bench_function("Generate all psuedo legal king moves", |b| {
        b.iter(|| generate_all_psuedo_legal_king_moves(&board))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(2000));
    targets = bench
}
criterion_main!(benches);
