mod boat;
mod cli;
mod database;

use std::{
    io::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use clap::Parser;
use whalecrab_engine::{platform_timer, timers::MoveTimer};

use crate::boat::Boat;

fn register_hooks(term: &Arc<AtomicBool>) -> Result<(), Error> {
    signal_hook::flag::register(signal_hook::consts::SIGINT, term.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, term.clone())?;
    Ok(())
}

fn main() {
    env_logger::init();

    // Log panics instead of printing to stderr
    std::panic::set_hook(Box::new(|e| log::debug!("{}", e)));

    let boat = {
        let args = cli::Args::parse();
        log::trace!("{:#?}", args);

        let positions = Arc::new(AtomicU64::new(0));
        let term = Arc::new(AtomicBool::new(false));
        let dataset = Arc::new(database::Dataset::load(&args.database_path));

        Boat {
            args,
            term,
            dataset,
            positions,
        }
    };

    for _ in 0..boat.args.threads {
        let boat = boat.clone();
        let _ = thread::spawn(move || {
            boat.sail();
        });
    }

    if let Err(e) = register_hooks(&boat.term) {
        log::error!(
            "Failed to register hooks. Program may appear unresponsive: {:?}",
            e
        );
    }

    let timer = platform_timer!(boat.args.time);
    while !timer.over() && !boat.term.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(5));
    }

    log::info!("Finishing program...");
    boat.term.store(true, Ordering::Relaxed);

    thread::sleep(Duration::from_millis(25));

    log::info!("Saving dataset to {}", boat.args.database_path.display());
    boat.dataset
        .save(&boat.args.database_path)
        .expect("Failed to save the dataset");
}
