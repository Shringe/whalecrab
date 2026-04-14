use std::{
    panic,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
};

use colored::Colorize;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use whalecrab_engine::{platform_timer, timers::MoveTimer};
use whalecrab_lib::position::game::{Game, State};

use crate::{
    cli::Args,
    database::{self, Dataset},
};

#[derive(Clone)]
pub struct Boat {
    pub args: Args,
    pub term: Arc<AtomicBool>,
    pub dataset: Arc<Dataset>,
    pub positions: Arc<AtomicU64>,
    pub errors: Arc<AtomicU32>,
}

impl Boat {
    pub fn sail(&self) {
        let mut seed = self.args.seed.unwrap_or_else(|| rand::rng().next_u32());
        log::trace!("Seed: {}", seed);
        let mut rng = SmallRng::seed_from_u64(seed.into());

        let mut game =
            Game::from_fen(&self.args.fen.clone().unwrap_or_default()).unwrap_or_default();

        let timer = platform_timer!(self.args.time);
        while !self.term.load(Ordering::Relaxed) {
            if timer.over() {
                log::debug!("Time is up");
                break;
            }

            let positions = self.positions.fetch_add(1, Ordering::Relaxed);
            log::trace!("Position #{}", positions);
            if positions >= self.args.positions {
                log::debug!("Maximum amount of positions reached");
                break;
            }

            macro_rules! save_position {
                ($error:expr) => {{
                    let errors = self.errors.fetch_add(1, Ordering::Relaxed);
                    log::trace!("Found error #{} at Position:\n{:#?}", errors, game);
                    if errors >= self.args.quit_after {
                        log::debug!("Maximum amount of errors reached");
                        break;
                    }

                    let (error, context) = $error.to_error_type_and_string();
                    let entry = database::Entry {
                        seed,
                        positions,
                        fen: game.to_fen(),
                        error,
                        context,
                    };

                    log::trace!("Adding entry to dataset: {:#?}", entry);
                    self.dataset.insert(seed, entry);
                }};
            }

            let moves = game.legal_moves();
            let Some(m) = moves.choose(&mut rng) else {
                log::trace!("No moves found. {:?}", game.state);
                if game.state == State::InProgress {
                    log::trace!("Game finished and was in progress!");
                    save_position!(database::ErrorInfo::FinishedInProgress {
                        state: format!("{:?}", game.state)
                    });
                }
                log::trace!("Starting new game");
                game = Game::default();
                continue;
            };

            log::trace!("Playing move: {:?}", m);
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                game.play(m);
            }));

            let Err(e) = result else {
                continue;
            };

            log::trace!("Found error {:?}", e);
            let logs = game.retrieve_logs();
            log::debug!("Recent logs:\n{}", logs.red());

            save_position!(database::ErrorInfo::PanicOnMove {
                m: m.to_string(),
                uci: m.to_uci(&game),
                error: format!("{:?}", e)
            });

            game = Game::default();
            seed = rand::rng().next_u32();
            log::trace!("New Seed: {}", seed);
            rng = SmallRng::seed_from_u64(seed.into());
        }

        self.term.store(true, Ordering::Relaxed);
        log::trace!("Quiting on seed {}", seed);
    }
}
