mod command;
mod interface;
mod logging;
#[cfg(test)]
mod test_utils;

use std::io;

use crate::interface::UciInterface;
use crate::logging::logger::Logger;

#[cfg(debug_assertions)]
fn replay_mode() {
    use std::io::Write;
    use std::{
        env, fs,
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    let args: Vec<String> = env::args().collect();

    if args.get(1) != Some(&"--replay".to_string()) {
        return;
    }
    let Some(path) = args.get(2) else {
        return;
    };

    let path: &Path = path.as_ref();
    assert!(
        path.exists(),
        "Tried to enter replay mode, but {} does not exist",
        path.display()
    );

    let file = fs::File::open(path).unwrap();

    let mut child = Command::new(&args[0])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();

    let lines = BufReader::new(file).lines().map_while(Result::ok);
    let input = std::io::stdin();

    for line in lines {
        let mut buf = String::new();
        let _ = input.read_line(&mut buf);
        writeln!(stdin, "{}", line).unwrap();
        stdin.flush().unwrap();
    }

    std::process::exit(0);
}

fn main() {
    #[cfg(debug_assertions)]
    replay_mode();

    #[allow(clippy::default_constructed_unit_structs)]
    let _g = Logger::default();

    let mut uci = UciInterface::default();

    let stdin = io::stdin();
    uci.watch(stdin);
}
