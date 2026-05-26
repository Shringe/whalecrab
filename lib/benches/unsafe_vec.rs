mod common;

use criterion::Criterion;
use whalecrab_lib::{square::Square, vectors::UnsafeVec};

fn bench(c: &mut Criterion) {
    let item = Square::E5;
    let capacity = 10_000;

    c.bench_function("Pushing to Vec", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(capacity);
            for _ in 0..capacity {
                v.push(item);
            }
        })
    });

    c.bench_function("Pushing to UnsafeVec", |b| {
        b.iter(|| {
            let mut uv = UnsafeVec::with_capacity(capacity);
            for _ in 0..capacity {
                unsafe { uv.push_unchecked(item) };
            }
        })
    });
}

setup_criterion!();
