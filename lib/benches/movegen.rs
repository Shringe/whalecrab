mod common;
use criterion::Criterion;
use whalecrab_lib::game::Game;

fn generate_all_psuedo_legal_pawn_moves(game: &mut Game) {
    for sq in game.white_pawns | game.black_pawns {
        sq.pawn_psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_rook_moves(game: &mut Game) {
    for sq in game.white_rooks | game.black_rooks {
        sq.rook_psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_knight_moves(game: &mut Game) {
    for sq in game.white_knights | game.black_knights {
        sq.knight_psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_bishop_moves(game: &mut Game) {
    for sq in game.white_bishops | game.black_bishops {
        sq.bishop_psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_queen_moves(game: &mut Game) {
    for sq in game.white_queens | game.black_queens {
        sq.queen_psuedo_legal_moves(game);
    }
}

fn generate_all_psuedo_legal_king_moves(game: &mut Game) {
    for sq in game.white_kings | game.black_kings {
        sq.king_psuedo_legal_moves(game);
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

setup_criterion!();
