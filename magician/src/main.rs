use std::fs;

use clap::Parser;
use magician::{embed_magic_rooks, generate_magic_rooks};
use whalecrab_lib::position::generator::GameGenerator;

mod cli;

fn main() {
    let args = cli::Args::parse();
    eprintln!("{:#?}", args);

    let mut grng = args
        .seed
        .map(GameGenerator::seeded)
        .unwrap_or(GameGenerator::unseeded());

    println!("Generating magic rooks...");
    let rooks = generate_magic_rooks(&mut grng, 12..=12);

    println!("Embedding magic rooks into source code...");
    let embedded = embed_magic_rooks(rooks);

    println!("Writing source code to {}", args.path.display());
    fs::write(&args.path, embedded).expect("Failed to write magic rooks");

    println!("Finished writing magic rooks to {}", args.path.display());
}
