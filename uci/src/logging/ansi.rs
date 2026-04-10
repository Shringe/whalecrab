/// Returns the color back to default
pub(crate) const FINISH: &str = "\x1b[0m";

// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
// All text after one of these colors will be colored
pub(crate) const YELLOW: &str = "\x1b[33m";
pub(crate) const MAGENTA: &str = "\x1b[35m";
pub(crate) const CYAN: &str = "\x1b[36m";

/// Colors a piece of text
pub(crate) fn color(text: &str, color: &str) -> String {
    format!("{color}{text}{FINISH}")
}
