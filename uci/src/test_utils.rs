/// Creates a UciCommand from a formatted string. Unwraps parsing errors.
/// ```rust
/// let mut uci = UciInterface::default();
/// let _ = uci.handle(uci!("uci"));
/// let _ = uci.handle(uci!("go movetime {}", 5000));
/// ```
#[macro_export]
macro_rules! uci {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::command::UciCommand::from_str(s.as_str()).expect(format!("Failed to parse uci!({})", s).as_str())
    }};
}
