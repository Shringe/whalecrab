mod common;

use std::hint::black_box;

use criterion::Criterion;
use whalecrab_lib::{
    movegen::{
        legal_moves::LegalMovesFilter,
        pieces::{bishop, king, knight, pawn, piece::PieceColor, queen, rook},
    },
    position::game::Game,
    vectors::UnsafeVec,
};

fn bench_game(c: &mut Criterion, group_name: &str, game: Game) {
    assert_eq!(
        game.turn,
        PieceColor::White,
        "This benchmark is setup for white"
    );

    let kingless_bb = game.occupied ^ game.black_kings;
    let enemy_occupied = game.black_occupied;

    let mut moves = UnsafeVec::with_capacity(game.maximum_move_count_white() as usize);

    let mut group = c.benchmark_group(group_name);

    let m = game.find_first_legal_move_white();
    println!(
        "First legal move: {:?} {:?}",
        m.map(|m| game.piece_lookup(m.from(game.turn)).unwrap().0),
        m
    );
    group.bench_function("Find first legal move", |b| {
        b.iter(|| {
            let _ = game.find_first_legal_move_white();
        });
    });

    let m = game.lazy_psuedo_legal_moves_white().next();
    println!(
        "First legal move: {:?} {:?}",
        m.map(|m| game.piece_lookup(m.from(game.turn)).unwrap().0),
        m
    );
    group.bench_function("Find first psuedo legal move", |b| {
        b.iter(|| {
            let _ = game.lazy_psuedo_legal_moves_white().next();
        });
    });

    group.bench_function("pawns", |b| {
        b.iter(|| {
            pawn::push_psuedo_legal_moves_white(&mut moves, &game);
            moves.clear();
        });
    });

    group.bench_function("knights", |b| {
        b.iter(|| {
            knight::push_psuedo_legal_moves(&mut moves, &game, game.white_knights, enemy_occupied);
            moves.clear();
        });
    });

    group.bench_function("bishops", |b| {
        b.iter(|| {
            bishop::push_psuedo_legal_moves(
                &mut moves,
                &game,
                game.white_bishops,
                kingless_bb,
                enemy_occupied,
            );
            moves.clear();
        });
    });

    group.bench_function("rooks", |b| {
        b.iter(|| {
            rook::push_psuedo_legal_moves(
                &mut moves,
                &game,
                game.white_rooks,
                kingless_bb,
                enemy_occupied,
            );
            moves.clear();
        });
    });

    group.bench_function("queens", |b| {
        b.iter(|| {
            queen::push_psuedo_legal_moves(
                &mut moves,
                &game,
                game.white_queens,
                kingless_bb,
                enemy_occupied,
            );
            moves.clear();
        });
    });

    group.bench_function("kings", |b| {
        b.iter(|| {
            king::push_psuedo_legal_moves(&mut moves, &game, game.white_kings, enemy_occupied);
            moves.clear();
        });
    });

    group.bench_function("castling", |b| {
        b.iter(|| {
            king::push_psuedo_legal_castling_moves_white(&mut moves, &game);
            moves.clear();
        });
    });

    group.finish();

    black_box(moves.finish());
}

fn bench_midgame(c: &mut Criterion) {
    bench_game(c, "midgame", common::midgame());
}

fn bench_earlygame(c: &mut Criterion) {
    bench_game(c, "earlygame", common::earlygame());
}

fn bench_lategame(c: &mut Criterion) {
    bench_game(c, "lategame", common::lategame());
}

fn bench(c: &mut Criterion) {
    let game = common::only_pawn_moves();

    let m = game.find_first_legal_move_white();
    println!(
        "First legal move: {:?} {:?}",
        m.map(|m| game.piece_lookup(m.from(game.turn)).unwrap().0),
        m
    );
    c.bench_function("Find first move when only pawn moves are available", |b| {
        b.iter(|| {
            let _ = game.find_first_legal_move_white();
        });
    });

    c.bench_function("Construct LegalMovesFilter", |b| {
        b.iter(|| {
            let _ = LegalMovesFilter::new(&game);
        });
    });
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion();
    targets = bench, bench_midgame, bench_earlygame, bench_lategame
}
criterion::criterion_main!(benches);
