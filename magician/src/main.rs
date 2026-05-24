mod cli;

use std::fs;

use clap::Parser;
use magician::rooks::{embedded_magic_rook_file, generate_magic_rooks};
use whalecrab_lib::position::generator::GameGenerator;

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
    let source = embedded_magic_rook_file(rooks);

    println!("Writing source code to {}", args.path.display());
    fs::write(&args.path, source).expect("Failed to write magic rooks");

    println!("Finished writing magic rooks to {}", args.path.display());
}
