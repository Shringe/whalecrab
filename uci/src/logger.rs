use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{Mutex, OnceLock},
};

static LOG_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::Logger::log(&(format!($($arg)*) + "\n"))
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
        Self::init("/tmp/whalecrab_uci.log")
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        log!("Flushing log file");
        if let Some(writer) = LOG_WRITER.get()
            && let Ok(mut w) = writer.lock()
        {
            match w.flush() {
                Ok(_) => eprintln!("Log flushed successfully"),
                Err(e) => eprintln!("Failed to flush log file: {}", e),
            }
        }
    }
}

impl Logger {
    pub fn init(path: &str) -> Self {
        match File::create(path) {
            Ok(f) => match LOG_WRITER.set(Mutex::new(BufWriter::new(f))) {
                Ok(_) => log!("Initialized logger at {}", path),
                Err(e) => {
                    let prior = e
                        .into_inner()
                        .ok()
                        .and_then(|m| m.into_inner().ok())
                        .map(|_| path)
                        .unwrap_or("unknown");
                    eprintln!("Logger was already initialized at {}", prior);
                }
            },
            Err(e) => eprintln!("Can't log to file: {}", e),
        }
        Self
    }

    fn log_with_prefix(prefix: &str, msg: &str) {
        eprint!("{}: {}", prefix, msg);
        if let Some(writer) = LOG_WRITER.get()
            && let Ok(mut w) = writer.lock()
            && let Err(e) = w.write_all(msg.as_bytes())
        {
            eprintln!("Couldn't write to log buffer: {}", e);
        }
    }

    pub fn log(msg: &str) {
        Self::log_with_prefix("LOGGER", msg);
    }
}
