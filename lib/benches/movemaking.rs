mod common;
use criterion::Criterion;

fn bench(c: &mut Criterion) {
    let mut game = common::midgame();
    let moves = game.generate_all_legal_moves();
    c.bench_function("Make/Unmake all legal moves", |b| {
        b.iter(|| {
            for m in &moves {
                game.play(m);
                game.unplay(m);
            }
        });
    });
}

setup_criterion!();
