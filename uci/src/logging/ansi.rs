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

pub const ALL_COLORS: [&str; 8] = [BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, WHITE];
pub const EXTRA_CODES: [&str; 1] = [FINISH];

/// Colors a piece of text
fn color(text: String, color: &str) -> String {
    format!("{color}{text}{FINISH}")
}

macro_rules! colorize {
    ($method:ident,  $color:expr) => {
        fn $method(&self) -> String {
            color(self.to_string(), $color)
        }
    };
}

pub trait Colorize: ToString {
    colorize!(black, BLACK);
    colorize!(red, RED);
    colorize!(green, GREEN);
    colorize!(yellow, YELLOW);
    colorize!(blue, BLUE);
    colorize!(magenta, MAGENTA);
    colorize!(cyan, CYAN);
    colorize!(white, WHITE);

    /// Removes the color from a string
    fn decolorize(&self) -> String {
        let mut out = self.to_string();
        for c in ALL_COLORS {
            out = out.replace(c, "");
        }
        for c in EXTRA_CODES {
            out = out.replace(c, "");
        }
        out
    }
}

impl Colorize for str {}
impl Colorize for String {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decolorize() {
        let s = "Hello, World!";
        let colored = s.blue();
        let actual = colored.decolorize();
        let expected = s.to_string();
        assert_eq!(actual, expected);
    }
}
