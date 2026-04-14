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
    time::{Duration, Instant},
};

use clap::Parser;
use colored::Colorize;

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

    let handles: Vec<_> = (1..boat.args.threads)
        .map(|_| {
            let boat = boat.clone();
            thread::spawn(move || {
                boat.sail();
            })
        })
        .collect();

    if let Err(e) = register_hooks(&boat.term) {
        log::error!(
            "Failed to register hooks. Program may appear unresponsive: {:?}",
            e
        );
    }

    let start = Instant::now();
    boat.sail();
    let _ = std::panic::take_hook();

    log::info!("Finishing program...");
    for handle in handles {
        let _ = handle.join();
    }

    let positions = boat.positions.load(Ordering::Relaxed);
    let errors = boat.errors.load(Ordering::Relaxed);
    display_results(positions, errors, start.elapsed());

    log::info!("Saving dataset to {}", boat.args.database_path.display());
    boat.dataset
        .save(&boat.args.database_path)
        .expect("Failed to save the dataset");
}

fn display_results(positions: u64, errors: u32, time_spent: Duration) {
    let quadrillion = 1_000_000_000_000_000;
    let trillion = 1_000_000_000_000;
    let billion = 1_000_000_000;
    let million = 1_000_000;
    let thousand = 1000;

    let per_quadrillion = (errors as u128 * quadrillion) / positions as u128;

    let line = if per_quadrillion >= trillion {
        format!(
            "Number of errors per {} positions:     {}",
            "thousand".red(),
            per_quadrillion / trillion
        )
    } else if per_quadrillion >= billion {
        format!(
            "Number of errors per {} positions:      {}",
            "million".yellow(),
            per_quadrillion / billion
        )
    } else if per_quadrillion >= million {
        format!(
            "Number of errors per {} positions:      {}",
            "billion".green(),
            per_quadrillion / million
        )
    } else if per_quadrillion >= thousand {
        format!(
            "Number of errors per {} positions:     {}",
            "trillion".cyan(),
            per_quadrillion / thousand
        )
    } else {
        format!(
            "Number of errors per {} positions:  {}",
            "quadrillion".bright_cyan(),
            per_quadrillion
        )
    };

    let elapsed = humantime::Duration::from(time_spent);
    log::info!("Total time spent searching:                  {}", elapsed);
    log::info!("Total number of positions searched:          {}", positions);
    log::info!("Total number of errors found:                {}", errors);
    log::info!("{}", line);
}
