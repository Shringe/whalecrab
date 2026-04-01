#[macro_export]
macro_rules! implement_operation {
    ($struct:ident, $trait:ident, $method:ident, $op:tt, Self, assign) => {
        impl std::ops::$trait for $struct {
            fn $method(&mut self, other: Self) {
                self.0 $op other.0;
            }
        }
    };
    ($struct:ident, $trait:ident, $method:ident, $op:tt, $type:ident, assign) => {
        impl std::ops::$trait<$type> for $struct {
            fn $method(&mut self, other: $type) {
                self.0 $op other;
            }
        }
    };
    ($struct:ident, $trait:ident, $method:ident, $op:tt, Self, new) => {
        impl std::ops::$trait for $struct {
            type Output = Self;
            fn $method(self, other: Self) -> Self {
                Self(self.0 $op other.0)
            }
        }
    };
    ($struct:ident, $trait:ident, $method:ident, $op:tt, $type:ident, new) => {
        impl std::ops::$trait<$type> for $struct {
            type Output = Self;
            fn $method(self, other: $type) -> Self {
                Self(self.0 $op other)
            }
        }
    };
}

/// Used to implement standard operations on wrapper types and their inner value.
///
/// ```rust
/// use whalecrab_lib::implement_operations;
///
/// #[derive(Debug, PartialEq, Clone, Copy)]
/// struct Score(i32);
///
/// // All suported operators for the parent type
/// implement_operations!(
///     Score,
///     Self,
///     [
///         Add,
///         AddAssign,
///         Sub,
///         SubAssign,
///         Mul,
///         Div,
///         BitAnd,
///         BitAndAssign,
///         BitOr,
///         BitOrAssign,
///         BitXor,
///         BitXorAssign,
///         Shl,
///         ShlAssign,
///         Shr,
///         ShrAssign
///     ]
/// );
///
/// // Implement a subset of operators for the inner type
/// implement_operations!(Score, i32, [Add, Sub]);
///
/// let mut score = Score(5);
/// score ^= Score(5); // Use BitOrAssign on the parent type
///
/// // Use Add on the inner and parent
/// assert_eq!(score + Score(5), score + 5);
/// ```
#[macro_export]
macro_rules! implement_operations {
    ($struct:ident, $type:ident, [$($op:ident),*]) => {
        $($crate::implement_operations!(@single $struct, $type, $op);)*
    };
    (@single $struct:ident, $type:ident, Add) => {
        $crate::implement_operation!($struct, Add, add, +, $type, new);
    };
    (@single $struct:ident, $type:ident, AddAssign) => {
        $crate::implement_operation!($struct, AddAssign, add_assign, +=, $type, assign);
    };
    (@single $struct:ident, $type:ident, Sub) => {
        $crate::implement_operation!($struct, Sub, sub, -, $type, new);
    };
    (@single $struct:ident, $type:ident, SubAssign) => {
        $crate::implement_operation!($struct, SubAssign, sub_assign, -=, $type, assign);
    };
    (@single $struct:ident, $type:ident, Mul) => {
        $crate::implement_operation!($struct, Mul, mul, *, $type, new);
    };
    (@single $struct:ident, $type:ident, MulAssign) => {
        $crate::implement_operation!($struct, MulAssign, mul, *=, $type, new);
    };
    (@single $struct:ident, $type:ident, Div) => {
        $crate::implement_operation!($struct, Div, div, /, $type, new);
    };
    (@single $struct:ident, $type:ident, DivAssign) => {
        $crate::implement_operation!($struct, Div, div, /=, $type, new);
    };
    (@single $struct:ident, $type:ident, BitAnd) => {
        $crate::implement_operation!($struct, BitAnd, bitand, &, $type, new);
    };
    (@single $struct:ident, $type:ident, BitAndAssign) => {
        $crate::implement_operation!($struct, BitAndAssign, bitand_assign, &=, $type, assign);
    };
    (@single $struct:ident, $type:ident, BitOr) => {
        $crate::implement_operation!($struct, BitOr, bitor, |, $type, new);
    };
    (@single $struct:ident, $type:ident, BitOrAssign) => {
        $crate::implement_operation!($struct, BitOrAssign, bitor_assign, |=, $type, assign);
    };
    (@single $struct:ident, $type:ident, BitXor) => {
        $crate::implement_operation!($struct, BitXor, bitxor, ^, $type, new);
    };
    (@single $struct:ident, $type:ident, BitXorAssign) => {
        $crate::implement_operation!($struct, BitXorAssign, bitxor_assign, ^=, $type, assign);
    };
    (@single $struct:ident, $type:ident, Shl) => {
        $crate::implement_operation!($struct, Shl, shl, <<, $type, new);
    };
    (@single $struct:ident, $type:ident, ShlAssign) => {
        $crate::implement_operation!($struct, ShlAssign, shl_assign, <<=, $type, assign);
    };
    (@single $struct:ident, $type:ident, Shr) => {
        $crate::implement_operation!($struct, Shr, shr, >>, $type, new);
    };
    (@single $struct:ident, $type:ident, ShrAssign) => {
        $crate::implement_operation!($struct, ShrAssign, shr_assign, >>=, $type, assign);
    };
}
