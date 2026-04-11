use std::{path::PathBuf, time::Duration};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(long)]
    pub fen: Option<String>,

    #[arg(long, default_value = "10s", value_parser=humantime::parse_duration)]
    pub time: Duration,

    #[arg(long, default_value_t = 100_000)]
    pub positions: u32,

    #[arg(long)]
    pub seed: Option<u32>,

    /// Quit after finding the first error
    #[arg(long)]
    pub quit: bool,

    /// Number of threads to use
    #[arg(long, default_value_t = 16)]
    pub threads: u8,

    /// Path to the database. You can set this to `/dev/null` to disable the db
    #[arg(long, default_value = "whalecrab_essex.csv")]
    pub database_path: PathBuf,
}
