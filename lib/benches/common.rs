#![allow(dead_code)]
use std::time::Duration;

use criterion::Criterion;
use whalecrab_lib::position::game::Game;

pub const EARLYGAME_FEN: &str = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
pub const MIDGAME_FEN: &str = "r1bq1rk1/ppp2ppp/2n2n2/2bp4/2B1P3/3P1N2/PPP2PPP/RNBQR1K1 w - - 0 8";
pub const LATEGAME_FEN: &str = "8/8/8/8/3k4/8/3P4/3K4 w - - 0 50";

pub fn earlygame() -> Game {
    Game::from_fen(EARLYGAME_FEN).unwrap()
}

pub fn midgame() -> Game {
    Game::from_fen(MIDGAME_FEN).unwrap()
}

pub fn lategame() -> Game {
    Game::from_fen(LATEGAME_FEN).unwrap()
}

pub fn configured_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_secs(2))
        .noise_threshold(0.05)
}

#[macro_export]
macro_rules! setup_criterion {
    () => {
        criterion::criterion_group! {
            name = benches;
            config = common::configured_criterion();
            targets = bench
        }
        criterion::criterion_main!(benches);
    };
}
