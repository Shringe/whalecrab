use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    sync::{Mutex, OnceLock},
};

use crate::logging::ansi;

static MAIN_ANSI_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static MAIN_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static SENT_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static RECEIVED_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logging::logger::Logger::log(&(format!($($arg)*) + "\n"))
    };
}

#[macro_export]
macro_rules! received {
    ($($arg:tt)*) => {
        $crate::logging::logger::Logger::received(&(format!($($arg)*) + "\n"))
    };
}

#[macro_export]
macro_rules! sent {
    ($($arg:tt)*) => {
        $crate::logging::logger::Logger::sent(&(format!($($arg)*) + "\n"))
    };
}

/// Flushes the logger any time an instance of this type is dropped.
/// You can prevent this type from being dropped immediately by using a binding.
/// ```rust
/// use uci::logger::Logger;
/// use uci::log;
///
/// {
///     // Logger is initialized and `let _g` binds the Logger to this scope
///     let _g = Logger::init("/dev/null");
///     log!("Hello, World!");
/// } // The scope ends and "Hello, World!" is flushed into the log file
///
/// ```
pub struct Logger;

impl Default for Logger {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let path = "/tmp/whalecrab_uci_debug";
        #[cfg(not(debug_assertions))]
        let path = "/tmp/whalecrab_uci";
        Self::init(path)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        log!("Flushing log files");
        for (writer, name) in [
            (MAIN_WRITER.get(), "main.log"),
            (SENT_WRITER.get(), "sent.log"),
            (RECEIVED_WRITER.get(), "received.log"),
            (MAIN_ANSI_WRITER.get(), "main.ans"),
        ] {
            if let Some(w) = writer
                && let Ok(mut w) = w.lock()
            {
                match w.flush() {
                    Ok(_) => eprintln!("{} flushed successfully", name),
                    Err(e) => eprintln!("Failed to flush {}: {}", name, e),
                }
            }
        }
    }
}

impl Logger {
    pub fn init<P: AsRef<Path>>(dir: P) -> Self {
        let dir: &Path = dir.as_ref();

        if !dir.exists()
            && let Err(e) = fs::create_dir(dir)
        {
            eprintln!("Can't create log directory: {}", e);
            return Self;
        }

        let init_writer = |writer: &OnceLock<Mutex<BufWriter<File>>>, filename: &str| {
            let path = dir.join(filename);
            match File::create(&path) {
                Ok(f) => {
                    if let Err(e) = writer.set(Mutex::new(BufWriter::new(f))) {
                        eprintln!("Logger {} was already initialized, {:?}", filename, e);
                    }
                }
                Err(e) => eprintln!("Can't open {}: {}", path.display(), e),
            }
        };

        init_writer(&MAIN_WRITER, "main.log");
        init_writer(&SENT_WRITER, "sent.log");
        init_writer(&RECEIVED_WRITER, "received.log");
        init_writer(&MAIN_ANSI_WRITER, "main.ans");

        log!("Initialized logger at {}", dir.display());
        Self
    }

    fn write_to(writer: &OnceLock<Mutex<BufWriter<File>>>, msg: &str) {
        if let Some(w) = writer.get()
            && let Ok(mut w) = w.lock()
            && let Err(e) = w.write_all(msg.as_bytes())
        {
            eprintln!("Couldn't write to log buffer: {}", e);
        }
    }

    fn log_with_prefix(prefix: &str, prefix_color: &str, msg: &str) {
        let text_prefix = prefix;
        let ansi_prefix = ansi::color(prefix, prefix_color);
        let text_prefixed = format!("{}: {}", text_prefix, msg);
        let ansi_prefixed = format!("{}: {}", ansi_prefix, msg);
        eprint!("{}", ansi_prefixed);
        Self::write_to(&MAIN_WRITER, &text_prefixed);
        Self::write_to(&MAIN_ANSI_WRITER, &ansi_prefixed);
    }

    pub fn log(msg: &str) {
        Self::log_with_prefix("Logger", ansi::GREEN, msg);
    }

    pub fn received(msg: &str) {
        Self::log_with_prefix("Received", ansi::CYAN, msg);
        Self::write_to(&RECEIVED_WRITER, msg);
    }

    pub fn sent(msg: &str) {
        Self::log_with_prefix("Sent", ansi::MAGENTA, msg);
        Self::write_to(&SENT_WRITER, msg);
    }
}
