use std::fmt;
use whalecrab_lib::implement_operations;

#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Score(i32);

implement_operations!(Score, Self, i32, [Add, AddAssign, Sub, SubAssign]);

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pawns = self.0 / 100;
        let centipawns = self.0 % 100;
        write!(f, "{}.{}", pawns, centipawns.abs())
    }
}

impl Score {
    pub const MAX: Score = Score(i32::MAX);
    pub const MIN: Score = Score(i32::MIN);

    pub const fn new(value: i32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Score::new(52019).to_string(), "520.19".to_string());
        assert_eq!(Score::new(-52019).to_string(), "-520.19".to_string());
    }
}
