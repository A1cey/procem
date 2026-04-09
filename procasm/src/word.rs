//! The [`ProcasmWord`] trait, its super traits and its implementations for all signed integer types.

use core::num::ParseIntError;

use procem::word::{I8, I16, I32, I64, I128, ISize, Word};

pub trait ProcasmWord: Word {
    /// This is a wrapper around the [`from_str_radix()`](i32::from_str_radix()) function that is implemented for all of Rust's numeric types.
    ///
    /// # Errors
    /// Returns [`ParseIntError`] when the parsing failed.
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError>;

    /// Checks for carry when adding.
    #[must_use]
    fn check_carry_add(&self, rhs: Self) -> bool;

    /// Checks for carry when subtracting.
    #[must_use]
    fn check_carry_sub(&self, rhs: Self) -> bool;

    /// Checks for carry when multiplying.
    #[must_use]
    fn check_carry_mul(&self, rhs: Self) -> bool;

    /// Checks for division overflow (i.e., MIN / -1 for signed types).
    /// Similiar to [`ProcasmWord::overflowing_div()`] this is a convenience wrapper over Rust's [`overflowing_div()`](i32::overflowing_div()).
    /// However it discards the result of the division.
    #[must_use]
    fn check_carry_div(&self, rhs: Self) -> bool;

    /// Convenience wrapper over Rust's [`overflowing_add()`](i32::overflowing_add()).
    #[must_use]
    fn overflowing_add(&self, rhs: Self) -> (Self, bool);
    /// Convenience wrapper over Rust's [`overflowing_sub()`](i32::overflowing_sub()).
    #[must_use]
    fn overflowing_sub(&self, rhs: Self) -> (Self, bool);
    /// Convenience wrapper over Rust's [`overflowing_mul()`](i32::overflowing_mul()).
    #[must_use]
    fn overflowing_mul(&self, rhs: Self) -> (Self, bool);
    /// Convenience wrapper over Rust's [`overflowing_div()`](i32::overflowing_div()).
    #[must_use]
    fn overflowing_div(&self, rhs: Self) -> (Self, bool);

    /// Convenience wrapper over Rust's [`rotate_left()`](i32::rotate_left()).
    #[must_use]
    fn rotate_left(&self, val: u32) -> Self;
    /// Convenience wrapper over Rust's [`rotate_right()`](i32::rotate_right()).
    #[must_use]
    fn rotate_right(&self, val: u32) -> Self;

    #[must_use]
    fn max() -> Self;
}

// Implements the ProcasmWord trait for the procem Word wrapper structs.
macro_rules! impl_word {
    ($name: ident, $type: ty $(,)? ) => {
        impl ProcasmWord for $name {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
                <$type>::from_str_radix(s, radix).map(|val| $name::from(val))
            }

            fn check_carry_add(&self, rhs: Self) -> bool {
                let (lhs, rhs) = (usize::from(*self) as u128, usize::from(rhs) as u128);
                lhs + rhs > <$type>::MAX as u128
            }

            fn check_carry_sub(&self, rhs: Self) -> bool {
                let (lhs, rhs) = (usize::from(*self) as u128, usize::from(rhs) as u128);
                lhs < rhs
            }

            fn check_carry_mul(&self, rhs: Self) -> bool {
                let (lhs, rhs) = (usize::from(*self) as u128, usize::from(rhs) as u128);
                lhs * rhs > <$type>::MAX as u128
            }

            fn check_carry_div(&self, rhs: Self) -> bool {
                <$type>::overflowing_div(<$type>::from(*self), <$type>::from(rhs)).1
            }

            fn overflowing_add(&self, rhs: Self) -> (Self, bool) {
                let (res, overflow) = <$type>::overflowing_add(<$type>::from(*self), <$type>::from(rhs));
                (Self::from(res), overflow)
            }

            fn overflowing_sub(&self, rhs: Self) -> (Self, bool) {
                let (res, overflow) = <$type>::overflowing_sub(<$type>::from(*self), <$type>::from(rhs));
                (Self::from(res), overflow)
            }

            fn overflowing_mul(&self, rhs: Self) -> (Self, bool) {
                let (res, overflow) = <$type>::overflowing_mul(<$type>::from(*self), <$type>::from(rhs));
                (Self::from(res), overflow)
            }

            fn overflowing_div(&self, rhs: Self) -> (Self, bool) {
                let (res, overflow) = <$type>::overflowing_div(<$type>::from(*self), <$type>::from(rhs));
                (Self::from(res), overflow)
            }

            fn rotate_left(&self, val: u32) -> Self {
                Self::from(<$type>::rotate_left(<$type>::from(*self), val))
            }

            fn rotate_right(&self, val: u32) -> Self {
                Self::from(<$type>::rotate_right(<$type>::from(*self), val))
            }

            fn max() -> Self {
                Self::from(<$type>::MAX)
            }
        }
    };
}

impl_word!(I8, i8);
impl_word!(I16, i16);
impl_word!(I32, i32);
impl_word!(I64, i64);
impl_word!(I128, i128);
impl_word!(ISize, isize);
