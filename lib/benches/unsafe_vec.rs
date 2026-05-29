mod common;

use std::hint::black_box;

use criterion::Criterion;
use whalecrab_lib::{
    square::Square,
    vectors::{ArrayVec, UnsafeVec, Vector},
};

fn bench_with_capacity<const C: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("{C} items"));

    let item = Square::E5.index();
    let capacity = C;
    black_box(capacity);

    group.bench_function("Pushing to Vec", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(capacity);
            for _ in 0..capacity {
                v.push(item);
            }
            black_box(v);
        })
    });

    group.bench_function("Pushing to UnsafeVec", |b| {
        b.iter(|| {
            let mut uv = UnsafeVec::with_capacity(capacity);
            for _ in 0..capacity {
                unsafe { uv.push_unchecked(item) };
            }
            let v = uv.finish();
            black_box(v);
        })
    });

    group.bench_function("Pushing to ArrayVec", |b| {
        b.iter(|| {
            let mut av = ArrayVec::<_, C>::new();
            for _ in 0..capacity {
                av.push(item);
            }
            black_box(av);
        })
    });
}

fn bench(c: &mut Criterion) {
    let game = common::midgame();

    c.bench_function("Generating moves with Vec", |b| {
        b.iter(|| {
            let mut moves = Vec::with_capacity(game.maximum_move_count_white() as usize);
            game.push_psuedo_legal_moves_white(&mut moves);
            black_box(moves);
        })
    });

    c.bench_function("Generating moves with UnsafeVec", |b| {
        b.iter(|| {
            let mut moves = UnsafeVec::with_capacity(game.maximum_move_count_white() as usize);
            game.push_psuedo_legal_moves_white(&mut moves);
            let moves = moves.finish();
            black_box(moves);
        })
    });

    c.bench_function("Generating moves with ArrayVec<512>", |b| {
        b.iter(|| {
            let mut moves = ArrayVec::<_, 512>::new();
            game.push_psuedo_legal_moves_white(&mut moves);
            black_box(moves);
        })
    });

    c.bench_function("Generating moves with ArrayVec<256>", |b| {
        b.iter(|| {
            let mut moves = ArrayVec::<_, 256>::new();
            game.push_psuedo_legal_moves_white(&mut moves);
            black_box(moves);
        })
    });

    bench_with_capacity::<256>(c);
    bench_with_capacity::<1024>(c);
    bench_with_capacity::<16>(c);
    bench_with_capacity::<128>(c);
    bench_with_capacity::<512>(c);
}

setup_criterion!();
