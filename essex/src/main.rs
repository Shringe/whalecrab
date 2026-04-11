mod boat;
mod cli;
mod database;

use std::{
    io::Error,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
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

    let args = cli::Args::parse();
    log::debug!("{:#?}", args);

    let term = Arc::new(AtomicBool::new(false));
    let dataset = Arc::new(Mutex::new(database::Dataset::load(&args.database_path)));

    for _ in 0..args.threads {
        let boat = Boat::new(&args, &term, &dataset);
        let _ = thread::spawn(move || {
            boat.sail();
        });
    }

    if let Err(e) = register_hooks(&term) {
        log::error!(
            "Failed to register hooks. Program may appear unresponsive: {:?}",
            e
        );
    }

    let timer = platform_timer!(args.time);
    while !timer.over() && !term.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(5));
    }

    log::info!("Finishing program...");
    term.store(true, Ordering::Relaxed);

    thread::sleep(Duration::from_millis(25));
    log::info!("Saving dataset to {}", args.database_path.display());
    dataset
        .lock()
        .expect("Failed to retrieve the dataset")
        .save(&args.database_path)
        .expect("Failed to save the dataset");
}
