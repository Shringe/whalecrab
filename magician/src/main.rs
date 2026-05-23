use std::fs;

use clap::Parser;
use magician::{MagicRooks, embed_magic_rooks, embedded_magic_rook_file, generate_magic_rooks};
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
    let source = embedded_magic_rook_file(rooks);
    // let mut source = String::with_capacity(std::mem::size_of::<MagicRooks>().saturating_mul(4));
    // source.push_str(&args.type_prefix);
    // source.push_str(" ROOKS: MagicRooks = [");
    // embed_magic_rooks(&mut source, rooks);
    // source.push_str("];");

    println!("Writing source code to {}", args.path.display());
    fs::write(&args.path, source).expect("Failed to write magic rooks");

    println!("Finished writing magic rooks to {}", args.path.display());
}
