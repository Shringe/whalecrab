mod cli;
mod database;

use std::panic;

use clap::Parser;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use whalecrab_engine::timers::{MoveTimer, elapsed::Elapsed};
use whalecrab_lib::position::game::{Game, STARTING_FEN, State};

fn play_game(args: &cli::Args) -> Option<database::Dataset> {
    let mut seed = args.seed.unwrap_or_else(|| rand::rng().next_u32());
    log::info!("Seed: {}", seed);
    let mut rng = SmallRng::seed_from_u64(seed.into());

    let mut game = Game::from_fen(&args.fen.clone().unwrap_or(STARTING_FEN.to_string()))?;
    let mut positions: u32 = 0;
    let timer = Elapsed::now(args.time);

    let mut db = database::Dataset::load(&args.database_path);

    macro_rules! save_position {
        ($last_move:expr, $last_move_uci:expr, $e:expr) => {
            log::debug!("Found error at Position:\n{:#?}", game);
            let entry = database::Entry {
                seed,
                positions,
                last_move: $last_move,
                last_move_uci: $last_move_uci,
                fen: game.to_fen(),
                error: format!("{:?}", $e),
            };

            log::debug!("Adding entry to dataset: {:#?}", entry);
            db.insert(seed, entry);
        };
    }

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

        let moves = game.legal_moves();
        let Some(m) = moves.choose(&mut rng) else {
            log::info!("No moves found. {:?}", game.state);
            if game.state == State::InProgress {
                log::warn!("Game finished and was in progress!");
                save_position!("None".to_string(), "None".to_string(), "FinishedInProgress");
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

        log::warn!("Found error {:?}", e);

        save_position!(m.to_string(), m.to_uci(&game), e);

        if args.quit {
            log::info!("Quiting");
            break;
        }

        game = Game::default();
        seed = rand::rng().next_u32();
        log::info!("New Seed: {}", seed);
        rng = SmallRng::seed_from_u64(seed.into());
    }

    log::info!("Seed: {}", seed);
    Some(db)
}

fn main() {
    env_logger::init();

    let args = cli::Args::parse();
    log::debug!("{:#?}", args);

    match play_game(&args) {
        Some(db) => match db.save(&args.database_path) {
            Ok(_) => log::info!("Database saved successfully"),
            Err(e) => log::error!("Failed to save database: {:?}", e),
        },
        None => log::error!("No database returned"),
    }
}
