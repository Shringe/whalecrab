use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Seed for magic finding
    #[arg(long)]
    pub seed: Option<u32>,

    /// Path to store the generated magic rooks source code
    #[arg(long, default_value = "/tmp/magic_rooks.rs")]
    pub path: PathBuf,
}
