mod cli;
mod database;

use std::panic;

use clap::Parser;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use whalecrab_engine::timers::{MoveTimer, elapsed::Elapsed};
use whalecrab_lib::position::game::{Game, STARTING_FEN, State};

fn play_game(args: &cli::Args) {
    let mut seed = args.seed.unwrap_or_else(|| rand::rng().next_u64());
    log::info!("Seed: {}", seed);
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut game = Game::from_fen(&args.fen.clone().unwrap_or(STARTING_FEN.to_string()))
        .expect("Provided fen is not valid");
    let mut positions: usize = 0;
    let timer = Elapsed::now(args.time);

    let mut db = database::load(&args.database_path);

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

        log::debug!("Position: {:#?}", game);
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
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            game.play(m);
        }));

        let Err(e) = result else {
            continue;
        };

        log::error!("Found error {:?}", e);
        log::error!("Saving error to db then restarting game");

        let entry = database::Entry { seed, positions };
        db.insert(seed, entry);
        if let Err(e) = database::save(&args.database_path, &db) {
            log::error!("Error saving database: {:?}", e);
        }

        if args.quit {
            log::info!("Quiting");
            break;
        }

        game = Game::default();
        seed = rand::rng().next_u64();
        log::info!("New Seed: {}", seed);
        rng = SmallRng::seed_from_u64(seed);
    }

    log::info!("Seed: {}", seed);
}

fn main() {
    env_logger::init();

    let args = cli::Args::parse();
    log::debug!("{:#?}", args);

    play_game(&args);
}
