mod boat;
mod cli;
mod database;

use std::{
    io::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use clap::Parser;

use crate::boat::Boat;

fn register_hooks(term: &Arc<AtomicBool>) -> Result<(), Error> {
    signal_hook::flag::register(signal_hook::consts::SIGINT, term.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, term.clone())?;
    Ok(())
}

fn main() {
    env_logger::init();

    // Log panics instead of printing to stderr
    std::panic::set_hook(Box::new(|e| log::warn!("{}", e)));

    let boat = {
        let args = cli::Args::parse();
        log::trace!("{:#?}", args);

        let term = Arc::new(AtomicBool::new(false));
        let dataset = Arc::new(database::Dataset::load(&args.database_path));
        let positions = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicU32::new(0));

        Boat {
            args,
            term,
            dataset,
            positions,
            errors,
        }
    };

    for _ in 1..boat.args.threads {
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

    boat.sail();
    let _ = std::panic::take_hook();
    log::info!("Finishing program...");

    thread::sleep(Duration::from_millis(50));
    let positions = boat.positions.load(Ordering::Relaxed);
    let errors = boat.errors.load(Ordering::Relaxed);
    let ratio = (errors as u64)
        .saturating_mul(1_000_000)
        .saturating_div(positions);
    log::info!("Total number of positions searched:      {}", positions);
    log::info!("Total number of errors found:            {}", errors);
    log::info!("Number of errors per million positions:  {}", ratio);

    log::info!("Saving dataset to {}", boat.args.database_path.display());
    boat.dataset
        .save(&boat.args.database_path)
        .expect("Failed to save the dataset");
}
