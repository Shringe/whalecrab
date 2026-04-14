pub struct BufLogger {
    buffer: [String; 256],
    counter: u8,
}

impl BufLogger {
    /// Create an empty log buffer
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            buffer: [const { String::new() }; 256],
            counter: 0,
        }
    }

    /// Push a message to the log buffer
    pub fn push(&mut self, msg: String) {
        unsafe { *self.buffer.get_unchecked_mut(self.counter as usize) = msg }
        self.counter = self.counter.wrapping_add(1);
    }

    /// Gets the last 256 log messages in chronological order
    pub fn retrieve(&self) -> String {
        let start = self.counter as usize;
        (0..256)
            .map(|i| self.buffer[(start + i) % 256].as_str())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_retrieve() {
        let mut logger = BufLogger::new();
        let expected = "1\n2\n3";

        logger.push("1".to_string());
        logger.push("2".to_string());
        logger.push("3".to_string());

        let actual = logger.retrieve();
        assert_eq!(actual, expected);
    }
}
