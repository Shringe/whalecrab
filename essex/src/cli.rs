use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub fen: Option<String>,

    #[arg(long, default_value = "10s", value_parser=humantime::parse_duration)]
    pub time: Duration,

    #[arg(long, default_value_t = 100_000)]
    pub positions: usize,

    #[arg(long)]
    pub seed: Option<u64>,
}
