pub mod ansi;

use std::{
    fs::{self, File},
    io::{BufWriter, Stdin, Write},
    path::Path,
    sync::{Mutex, OnceLock},
};

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logging::Logger::log(&(format!($($arg)*) + "\n"))
    };
}

#[macro_export]
macro_rules! received {
    ($($arg:tt)*) => {
        $crate::logging::Logger::received(&(format!($($arg)*) + "\n"))
    };
}

#[macro_export]
macro_rules! sent {
    ($($arg:tt)*) => {
        $crate::logging::Logger::sent(&(format!($($arg)*) + "\n"))
    };
}

/// Acts like a sent!() + println!(), but with special formatting if the interface is interactive
#[macro_export]
macro_rules! send {
    ($($arg:tt)*) => {
        let msg = format!($($arg)*);
        $crate::sent!("{}", msg);
        if let Some(interactive) = $crate::logging::INTERACTIVE.get()
            && *interactive
        {
            println!("{}", $crate::logging::ansi::color(&msg, $crate::logging::ansi::GREEN));
        } else {
            println!("{}", msg);
        };
    };
}

static MAIN_ANSI_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static MAIN_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static SENT_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static RECEIVED_WRITER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();

pub static INTERACTIVE: OnceLock<bool> = OnceLock::new();

pub fn check_for_interactive_session(stdin: &Stdin) {
    INTERACTIVE.get_or_init(move || {
        #[cfg(feature = "is-terminal")]
        {
            use is_terminal::IsTerminal;
            let out = stdin.is_terminal();
            log!("Interactive session detected");
            out
        }
        #[cfg(not(feature = "is-terminal"))]
        false
    });
}

/// Flush all log files
pub fn flush() {
    for (writer, name) in [
        (MAIN_WRITER.get(), "main.log"),
        (SENT_WRITER.get(), "sent.log"),
        (RECEIVED_WRITER.get(), "received.log"),
        (MAIN_ANSI_WRITER.get(), "main.ans"),
    ] {
        if let Some(w) = writer
            && let Ok(mut w) = w.lock()
            && let Err(e) = w.flush()
        {
            eprintln!("Failed to flush {}: {}", name, e);
        }
    }
}

/// Flushes the logger any time an instance of this type is dropped.
/// You can prevent this type from being dropped immediately by using a binding.
/// ```rust
/// use uci::logging::Logger;
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
        let parent = Path::new("/tmp/whalecrab");
        if !parent.exists() {
            if let Err(e) = fs::create_dir(parent) {
                eprintln!("Can't create log parent directory: {}", e);
                return Self;
            }
            // Relax permissions so other users can write into parent
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o777));
            }
        }

        #[cfg(debug_assertions)]
        let base = parent.join("debug");
        #[cfg(not(debug_assertions))]
        let base = parent.join("release");

        if !base.exists()
            && let Err(e) = fs::create_dir(&base)
        {
            eprintln!("Can't create log directory: {}", e);
            return Self;
        }

        for slot in u8::MIN..=u8::MAX {
            let path = base.join(slot.to_string());
            if path.exists() {
                continue;
            }
            eprintln!("Found slot for logger: {}", path.display());
            let slot_above = slot.wrapping_add(1);
            let above = base.join(slot_above.to_string());
            let _ = fs::remove_dir_all(&above);
            let _ = fs::create_dir(&path);

            #[cfg(unix)]
            {
                let last = base.join("last");
                if last.exists() {
                    let _ = fs::remove_file(&last);
                }
                let _ = std::os::unix::fs::symlink(&path, &last);
            }

            return Self::init(path);
        }

        let msg = "Failed to find an open slot for logger";
        if cfg!(debug_assertions) {
            panic!("{}", msg);
        } else {
            eprintln!("{}", msg);
            Self::init("/dev/null")
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        flush();
    }
}

impl Logger {
    pub fn init<P: AsRef<Path>>(dir: P) -> Self {
        let dir: &Path = dir.as_ref();

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
        {
            let _ = w.write_all(msg.as_bytes());
        }
    }

    fn log_with_prefix(prefix: &str, prefix_color: &str, msg: &str) {
        let text_prefix = prefix;
        let ansi_prefix = ansi::color(prefix, prefix_color);

        let prefix_lines = |msg: &str, prefix: &str| {
            msg.lines()
                .map(|line| format!("{}: {}", prefix, line))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        };

        let text_prefixed = prefix_lines(msg, text_prefix);
        let ansi_prefixed = prefix_lines(msg, &ansi_prefix);

        eprint!("{}", ansi_prefixed);
        Self::write_to(&MAIN_WRITER, &text_prefixed);
        Self::write_to(&MAIN_ANSI_WRITER, &ansi_prefixed);
    }

    pub fn log(msg: &str) {
        Self::log_with_prefix("Logger", ansi::YELLOW, msg);
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
