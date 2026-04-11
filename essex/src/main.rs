mod cli;

use clap::Parser;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use whalecrab_engine::timers::{MoveTimer, elapsed::Elapsed};
use whalecrab_lib::position::game::{Game, STARTING_FEN, State};

fn play_game(args: &cli::Args) {
    let seed = args.seed.unwrap_or_else(|| rand::rng().next_u64());
    log::info!("Seed: {}", seed);
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut game = Game::from_fen(&args.fen.clone().unwrap_or(STARTING_FEN.to_string()))
        .expect("Provided fen is not valid");
    let mut positions: usize = 0;
    let timer = Elapsed::now(args.time);

    loop {
        log::debug!("Position #{}", positions);
        positions = positions.saturating_add(1);
        if positions >= args.positions {
            log::info!("{} positions reached", args.positions);
            break;
        }
        if timer.over() {
            log::info!("Timer finished; {:?} is up", args.time);
            break;
        }

        log::debug!("Position: {:?}", game);
        let moves = game.legal_moves();
        let Some(m) = moves.choose(&mut rng) else {
            log::info!("No moves found. {:?}", game.state);
            if game.state == State::InProgress {
                log::error!("Game finished and was in progress!");
            }
            log::info!("Starting new game");
            game = Game::default();
            continue;
        };

        log::debug!("Playing move: {:?}", m);
        game.play(m);
    }

    log::info!("Seed: {}", seed);
}
fn main() {
    env_logger::init();

    let args = cli::Args::parse();
    log::debug!("{:#?}", args);

    play_game(&args);
}
