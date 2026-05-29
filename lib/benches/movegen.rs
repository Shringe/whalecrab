mod common;

use criterion::Criterion;
use whalecrab_lib::{
    movegen::{
        moves::Move,
        pieces::{
            self, pawn,
            piece::{ALL_PIECE_TYPES, PieceColor},
        },
    },
    vectors::{UnsafeVec, Vector},
};

macro_rules! bench_piece_method {
    ($c:expr, $game:expr, $type:expr, $method:ident) => {
        let squares = *$game.get_pieces(&$type, &PieceColor::White)
            | *$game.get_pieces(&$type, &PieceColor::Black);

        $c.bench_function(
            &format!(
                "Generate each {:?}.{} on the board",
                $type,
                stringify!($method),
            ),
            |b| {
                b.iter(|| {
                    for sq in squares {
                        $type.$method(&$game, &sq);
                    }
                })
            },
        );
    };
}

fn bench(c: &mut Criterion) {
    let mut game = common::midgame();

    macro_rules! bench_piece_methods {
        ($type:expr) => {
            bench_piece_method!(c, game, $type, psuedo_legal_moves);
            bench_piece_method!(c, game, $type, psuedo_legal_targets_fast);
            bench_piece_method!(c, game, $type, legal_moves);
        };
    }

    c.bench_function("Generate all legal moves", |b| {
        b.iter(|| game.legal_moves());
    });

    c.bench_function("Generate all psuedo legal moves", |b| {
        b.iter(|| game.generate_all_psuedo_legal_moves());
    });

    c.bench_function("Generate all legal moves lazily", |b| {
        b.iter(|| {
            let mut moves = UnsafeVec::with_capacity(game.maximum_move_count_white() as usize);
            for m in game.lazy_legal_moves_white() {
                moves.push(m);
            }
            let _ = moves.finish();
        });
    });

    c.bench_function("Generate all psuedo legal moves lazily", |b| {
        b.iter(|| {
            let mut moves = UnsafeVec::with_capacity(game.maximum_move_count_white() as usize);
            for m in game.lazy_psuedo_legal_moves_white() {
                moves.push(m);
            }
            let _ = moves.finish();
        });
    });

    let moves = game.generate_all_psuedo_legal_moves();
    c.bench_function("Filter for legal moves", |b| {
        b.iter(|| game.legal_moves_filter(moves.clone()));
    });

    c.bench_function("Generate grouped pawn moves", |b| {
        let capacity = game.white_pawns.popcnt() * pieces::pawn::MAXIMUM_MOVE_COUNT;
        b.iter(|| {
            let mut moves = UnsafeVec::with_capacity(capacity as usize);
            pawn::push_psuedo_legal_moves_white(&mut moves, &game);
            let _ = moves.finish();
        });
    });

    c.bench_function("Move inference / Constructing all moves", |b| {
        b.iter(|| {
            for m in &moves {
                Move::infer(m.from(game.turn), m.to(&game), &game);
            }
        })
    });

    for p in ALL_PIECE_TYPES {
        bench_piece_methods!(p);
    }
}

setup_criterion!();
