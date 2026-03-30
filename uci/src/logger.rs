use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{Mutex, OnceLock},
};

static LOG_WRITER: OnceLock<Mutex<LogWriter>> = OnceLock::new();

pub fn init(path: &str) {
    match File::create(path) {
        Ok(f) => {
            let _ = LOG_WRITER.set(Mutex::new(LogWriter(BufWriter::new(f))));
        }
        Err(e) => eprintln!("Can't log to file: {}", e),
    }
}

pub struct LogWriter(BufWriter<File>);

impl LogWriter {
    fn write(&mut self, msg: &str) {
        eprint!("LOGGER: {}", msg);
        if let Err(e) = self.0.write_all(msg.as_bytes()) {
            eprintln!("Couldn't write to log buffer: {}", e);
        }
    }

    pub fn log(msg: &str) {
        if let Some(writer) = LOG_WRITER.get() {
            if let Ok(mut w) = writer.lock() {
                w.write(msg);
            }
        } else {
            eprint!("LOGGER: {}", msg);
        }
    }
}

impl Drop for LogWriter {
    fn drop(&mut self) {
        match self.0.flush() {
            Ok(_) => eprintln!("Log flushed successfully"),
            Err(e) => eprintln!("Failed to flush log file: {}", e),
        }
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::LogWriter::log(&(format!($($arg)*) + "\n"))
    };
}
