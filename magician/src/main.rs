mod cli;

use std::fs;

use clap::Parser;
use magician::{
    bishops::{MagicBishop, embedded_magic_bishop_file, generate_magic_bishops},
    rooks::{MagicRook, embedded_magic_rook_file, generate_magic_rooks},
};
use whalecrab_lib::position::generator::GameGenerator;

fn main() {
    let args = cli::Args::parse();
    eprintln!("{:#?}", args);

    let mut grng = args
        .seed
        .map(GameGenerator::seeded)
        .unwrap_or(GameGenerator::unseeded());

    match args.mode {
        cli::Mode::Bishops => {
            println!("Generating magic bishops...");
            let bishops = generate_magic_bishops(&mut grng, MagicBishop::BIT_RANGE);

            println!("Embedding magic bishops into source code...");
            let source = embedded_magic_bishop_file(bishops);

            println!("Writing source code to {}", args.path.display());
            fs::write(&args.path, source).expect("Failed to write magic bishops");

            println!("Finished writing magic bishops to {}", args.path.display());
        }
        cli::Mode::Rooks => {
            println!("Generating magic rooks...");
            let rooks = generate_magic_rooks(&mut grng, MagicRook::BIT_RANGE);

            println!("Embedding magic rooks into source code...");
            let source = embedded_magic_rook_file(rooks);

            println!("Writing source code to {}", args.path.display());
            fs::write(&args.path, source).expect("Failed to write magic rooks");

            println!("Finished writing magic rooks to {}", args.path.display());
        }
    }
}
