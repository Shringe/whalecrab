use std::{
    panic,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
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
}

impl Boat {
    pub fn sail(&self) {
        let mut seed = self.args.seed.unwrap_or_else(|| rand::rng().next_u32());
        log::trace!("Seed: {}", seed);
        let mut rng = SmallRng::seed_from_u64(seed.into());

        let mut game =
            Game::from_fen(&self.args.fen.clone().unwrap_or_default()).unwrap_or_default();

        macro_rules! save_position {
            ($error:expr) => {
                log::debug!("Found error at Position:\n{:#?}", game);
                let (error, context) = $error.to_error_type_and_string();
                let entry = database::Entry {
                    seed,
                    positions: self.positions.load(Ordering::Relaxed),
                    fen: game.to_fen(),
                    error,
                    context,
                };

                log::trace!("Adding entry to dataset: {:#?}", entry);
                self.dataset.insert(seed, entry);
            };
        }

        while !self.term.load(Ordering::Relaxed) {
            {
                let positions = self.positions.fetch_add(1, Ordering::Relaxed);
                log::trace!("Position #{}", positions);
                if positions >= self.args.positions {
                    log::warn!("{} positions reached", self.args.positions);
                    self.term.store(true, Ordering::Relaxed);
                    break;
                }
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
