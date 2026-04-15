/// Returns the color back to default
pub const FINISH: &str = "\x1b[0m";

// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
// All text after one of these colors will be colored
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

/// Colors a piece of text
pub fn color(text: &str, color: &str) -> String {
    format!("{color}{text}{FINISH}")
}
