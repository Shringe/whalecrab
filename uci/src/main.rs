mod command;
mod interface;
mod logger;

use std::io;

use crate::interface::UciInterface;
use crate::logger::Logger;

fn main() {
    #[allow(clippy::default_constructed_unit_structs)]
    let _g = Logger::default();

    let mut uci = UciInterface::default();

    let stdin = io::stdin();
    uci.watch(stdin);
}
