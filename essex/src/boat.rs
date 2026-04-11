use std::{
    panic,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use whalecrab_lib::position::game::{Game, State};

use crate::{
    cli::Args,
    database::{self, Dataset},
};

pub struct Boat {
    args: Args,
    term: Arc<AtomicBool>,
    dataset: Arc<Mutex<Dataset>>,
}

impl Boat {
    pub fn new(args: &Args, term: &Arc<AtomicBool>, dataset: &Arc<Mutex<Dataset>>) -> Boat {
        Boat {
            args: args.clone(),
            term: term.clone(),
            dataset: dataset.clone(),
        }
    }

    pub fn sail(&self) {
        let mut seed = self.args.seed.unwrap_or_else(|| rand::rng().next_u32());
        log::trace!("Seed: {}", seed);
        let mut rng = SmallRng::seed_from_u64(seed.into());

        let mut game =
            Game::from_fen(&self.args.fen.clone().unwrap_or_default()).unwrap_or_default();
        let mut positions: u32 = 0;

        macro_rules! save_position {
            ($error:expr) => {
                log::debug!("Found error at Position:\n{:#?}", game);
                let (error, context) = $error.to_error_type_and_string();
                let entry = database::Entry {
                    seed,
                    positions,
                    fen: game.to_fen(),
                    error,
                    context,
                };

                log::trace!("Adding entry to dataset: {:#?}", entry);
                match self.dataset.lock() {
                    Ok(mut d) => {
                        let _ = d.insert(seed, entry);
                    }
                    Err(e) => log::error!("Failed write entry: {:?}", e),
                }
            };
        }

        while !self.term.load(Ordering::Relaxed) {
            log::trace!("Position #{}", positions);
            positions = positions.saturating_add(1);
            if positions >= self.args.positions {
                log::trace!("{} positions reached", self.args.positions);
                break;
            }
            let moves = game.legal_moves();
            let Some(m) = moves.choose(&mut rng) else {
                log::trace!("No moves found. {:?}", game.state);
                if game.state == State::InProgress {
                    log::debug!("Game finished and was in progress!");
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

            log::debug!("Found error {:?}", e);

            save_position!(database::ErrorInfo::PanicOnMove {
                m: m.to_string(),
                uci: m.to_uci(&game),
                error: format!("{:?}", e)
            });

            if self.args.quit {
                log::trace!("Quiting");
                break;
            }

            game = Game::default();
            seed = rand::rng().next_u32();
            log::trace!("New Seed: {}", seed);
            rng = SmallRng::seed_from_u64(seed.into());
        }

        log::trace!("Seed: {}", seed);
    }
}
