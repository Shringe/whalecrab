use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

macro_rules! implement_operation {
    ($trait:ident, $method:ident, $op:tt, Self, assign) => {
        impl $trait for Score {
            fn $method(&mut self, other: Self) {
                self.0 $op other.0;
            }
        }
    };
    ($trait:ident, $method:ident, $op:tt, $type:ident, assign) => {
        impl $trait<$type> for Score {
            fn $method(&mut self, other: $type) {
                self.0 $op other;
            }
        }
    };
    ($trait:ident, $method:ident, $op:tt, Self, new) => {
        impl $trait for Score {
            type Output = Self;
            fn $method(self, other: Self) -> Self {
                Self(self.0 $op other.0)
            }
        }
    };
    ($trait:ident, $method:ident,  $op:tt, $type:ident, new) => {
        impl $trait<$type> for Score {
            type Output = Self;
            fn $method(self, other: $type) -> Self {
                Self(self.0 $op other)
            }
        }
    };
}

macro_rules! implement_operations {
    (Self) => {
        implement_operation!(Add, add, +, Self, new);
        implement_operation!(AddAssign, add_assign, +=, Self, assign);
        implement_operation!(Sub, sub, -, Self, new);
        implement_operation!(SubAssign, sub_assign, -=, Self, assign);
    };
    ($type:ident) => {
        implement_operation!(Add, add, +, $type, new);
        implement_operation!(AddAssign, add_assign, +=, $type, assign);
        implement_operation!(Sub, sub, -, $type, new);
        implement_operation!(SubAssign, sub_assign, -=, $type, assign);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Score(i32);

implement_operations!(Self);
implement_operations!(i32);

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

    pub fn new(value: i32) -> Self {
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
