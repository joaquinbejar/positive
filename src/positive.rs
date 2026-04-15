/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Core implementation of the Positive type.

use crate::constants::EPSILON;
use crate::error::PositiveError;
use approx::{AbsDiffEq, RelativeEq};
use num_traits::{FromPrimitive, Pow, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{Ordering, PartialEq};
use std::fmt;
use std::fmt::Display;
#[cfg(not(feature = "non-zero"))]
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::str::FromStr;

/// A wrapper type that represents a guaranteed positive decimal value.
///
/// This type encapsulates a `Decimal` value and ensures through its API that
/// the contained value is always positive (greater than or equal to zero).
///
/// When the `non-zero` feature is enabled, the value must be strictly
/// greater than zero.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Positive(Decimal);

/// Returns whether the given decimal value satisfies the positivity constraint.
///
/// Without the `non-zero` feature, values >= 0 are accepted.
/// With the `non-zero` feature, only values > 0 are accepted.
#[inline]
#[must_use]
pub fn is_valid_positive_value(value: Decimal) -> bool {
    #[cfg(feature = "non-zero")]
    {
        value > Decimal::ZERO
    }
    #[cfg(not(feature = "non-zero"))]
    {
        value >= Decimal::ZERO
    }
}

/// Returns the minimum bound for error messages.
///
/// Without the `non-zero` feature, the minimum is 0.0.
/// With the `non-zero` feature, the minimum is the smallest representable
/// positive f64 value.
#[inline]
#[must_use]
fn min_bound() -> f64 {
    #[cfg(feature = "non-zero")]
    {
        f64::MIN_POSITIVE
    }
    #[cfg(not(feature = "non-zero"))]
    {
        0.0
    }
}

/// Determines if the given type parameter `T` is the `Positive` type.
#[must_use]
pub fn is_positive<T: 'static>() -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<Positive>()
}

impl Positive {
    // Re-export constants from the constants module for backward compatibility
    /// A zero value represented as a `Positive` value.
    ///
    /// This constant is not available when the `non-zero` feature is enabled.
    #[cfg(not(feature = "non-zero"))]
    pub const ZERO: Positive = crate::constants::ZERO;
    /// A value of one represented as a `Positive` value.
    pub const ONE: Positive = crate::constants::ONE;
    /// A value of two represented as a `Positive` value.
    pub const TWO: Positive = crate::constants::TWO;
    /// A value of three represented as a `Positive` value.
    pub const THREE: Positive = crate::constants::THREE;
    /// A value of four represented as a `Positive` value.
    pub const FOUR: Positive = crate::constants::FOUR;
    /// A value of five represented as a `Positive` value.
    pub const FIVE: Positive = crate::constants::FIVE;
    /// A value of six represented as a `Positive` value.
    pub const SIX: Positive = crate::constants::SIX;
    /// A value of seven represented as a `Positive` value.
    pub const SEVEN: Positive = crate::constants::SEVEN;
    /// A value of eight represented as a `Positive` value.
    pub const EIGHT: Positive = crate::constants::EIGHT;
    /// A value of nine represented as a `Positive` value.
    pub const NINE: Positive = crate::constants::NINE;
    /// A value of ten represented as a `Positive` value.
    pub const TEN: Positive = crate::constants::TEN;
    /// A value of fifteen represented as a `Positive` value.
    pub const FIFTEEN: Positive = crate::constants::FIFTEEN;
    /// A value of twenty represented as a `Positive` value.
    pub const TWENTY: Positive = crate::constants::TWENTY;
    /// A value of twenty-five represented as a `Positive` value.
    pub const TWENTY_FIVE: Positive = crate::constants::TWENTY_FIVE;
    /// A value of thirty represented as a `Positive` value.
    pub const THIRTY: Positive = crate::constants::THIRTY;
    /// A value of thirty-five represented as a `Positive` value.
    pub const THIRTY_FIVE: Positive = crate::constants::THIRTY_FIVE;
    /// A value of forty represented as a `Positive` value.
    pub const FORTY: Positive = crate::constants::FORTY;
    /// A value of forty-five represented as a `Positive` value.
    pub const FORTY_FIVE: Positive = crate::constants::FORTY_FIVE;
    /// A value of fifty represented as a `Positive` value.
    pub const FIFTY: Positive = crate::constants::FIFTY;
    /// A value of fifty-five represented as a `Positive` value.
    pub const FIFTY_FIVE: Positive = crate::constants::FIFTY_FIVE;
    /// A value of sixty represented as a `Positive` value.
    pub const SIXTY: Positive = crate::constants::SIXTY;
    /// A value of sixty-five represented as a `Positive` value.
    pub const SIXTY_FIVE: Positive = crate::constants::SIXTY_FIVE;
    /// A value of seventy represented as a `Positive` value.
    pub const SEVENTY: Positive = crate::constants::SEVENTY;
    /// A value of seventy-five represented as a `Positive` value.
    pub const SEVENTY_FIVE: Positive = crate::constants::SEVENTY_FIVE;
    /// A value of eighty represented as a `Positive` value.
    pub const EIGHTY: Positive = crate::constants::EIGHTY;
    /// A value of eighty-five represented as a `Positive` value.
    pub const EIGHTY_FIVE: Positive = crate::constants::EIGHTY_FIVE;
    /// A value of ninety represented as a `Positive` value.
    pub const NINETY: Positive = crate::constants::NINETY;
    /// A value of ninety-five represented as a `Positive` value.
    pub const NINETY_FIVE: Positive = crate::constants::NINETY_FIVE;
    /// A value of one hundred represented as a `Positive` value.
    pub const HUNDRED: Positive = crate::constants::HUNDRED;
    /// A value of two hundred represented as a `Positive` value.
    pub const TWO_HUNDRED: Positive = crate::constants::TWO_HUNDRED;
    /// A value of three hundred represented as a `Positive` value.
    pub const THREE_HUNDRED: Positive = crate::constants::THREE_HUNDRED;
    /// A value of four hundred represented as a `Positive` value.
    pub const FOUR_HUNDRED: Positive = crate::constants::FOUR_HUNDRED;
    /// A value of five hundred represented as a `Positive` value.
    pub const FIVE_HUNDRED: Positive = crate::constants::FIVE_HUNDRED;
    /// A value of six hundred represented as a `Positive` value.
    pub const SIX_HUNDRED: Positive = crate::constants::SIX_HUNDRED;
    /// A value of seven hundred represented as a `Positive` value.
    pub const SEVEN_HUNDRED: Positive = crate::constants::SEVEN_HUNDRED;
    /// A value of eight hundred represented as a `Positive` value.
    pub const EIGHT_HUNDRED: Positive = crate::constants::EIGHT_HUNDRED;
    /// A value of nine hundred represented as a `Positive` value.
    pub const NINE_HUNDRED: Positive = crate::constants::NINE_HUNDRED;
    /// A value of one thousand represented as a `Positive` value.
    pub const THOUSAND: Positive = crate::constants::THOUSAND;
    /// A value of two thousand represented as a `Positive` value.
    pub const TWO_THOUSAND: Positive = crate::constants::TWO_THOUSAND;
    /// A value of three thousand represented as a `Positive` value.
    pub const THREE_THOUSAND: Positive = crate::constants::THREE_THOUSAND;
    /// A value of four thousand represented as a `Positive` value.
    pub const FOUR_THOUSAND: Positive = crate::constants::FOUR_THOUSAND;
    /// A value of five thousand represented as a `Positive` value.
    pub const FIVE_THOUSAND: Positive = crate::constants::FIVE_THOUSAND;
    /// A value of six thousand represented as a `Positive` value.
    pub const SIX_THOUSAND: Positive = crate::constants::SIX_THOUSAND;
    /// A value of seven thousand represented as a `Positive` value.
    pub const SEVEN_THOUSAND: Positive = crate::constants::SEVEN_THOUSAND;
    /// A value of eight thousand represented as a `Positive` value.
    pub const EIGHT_THOUSAND: Positive = crate::constants::EIGHT_THOUSAND;
    /// A value of nine thousand represented as a `Positive` value.
    pub const NINE_THOUSAND: Positive = crate::constants::NINE_THOUSAND;
    /// A value of ten thousand represented as a `Positive` value.
    pub const TEN_THOUSAND: Positive = crate::constants::TEN_THOUSAND;
    /// The mathematical constant π (pi) represented as a `Positive` value.
    pub const PI: Positive = crate::constants::PI;
    /// The mathematical constant e (Euler's number) represented as a `Positive` value.
    pub const E: Positive = crate::constants::E;
    /// Represents the maximum positive value possible (effectively infinity).
    pub const INFINITY: Positive = crate::constants::INFINITY;

    /// Creates a new `Positive` value from a 64-bit floating-point number.
    ///
    /// Without the `non-zero` feature, values >= 0 are accepted.
    /// With the `non-zero` feature, only values > 0 are accepted.
    pub fn new(value: f64) -> Result<Self, PositiveError> {
        let dec = Decimal::from_f64(value);
        match dec {
            Some(value) if is_valid_positive_value(value) => Ok(Positive(value)),
            Some(value) => Err(PositiveError::OutOfBounds {
                value: value.to_f64().unwrap_or(0.0),
                min: min_bound(),
                max: f64::MAX,
            }),
            None => Err(PositiveError::ConversionError {
                from_type: "f64".to_string(),
                to_type: "Positive".to_string(),
                reason: "failed to parse Decimal".to_string(),
            }),
        }
    }

    /// Creates a new `Positive` value directly from a `Decimal`.
    ///
    /// Without the `non-zero` feature, values >= 0 are accepted.
    /// With the `non-zero` feature, only values > 0 are accepted.
    pub fn new_decimal(value: Decimal) -> Result<Self, PositiveError> {
        if is_valid_positive_value(value) {
            Ok(Positive(value))
        } else {
            Err(PositiveError::OutOfBounds {
                value: value.to_f64().unwrap_or(0.0),
                min: min_bound(),
                max: f64::INFINITY,
            })
        }
    }

    /// Returns the inner `Decimal` value.
    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` value (alias for `value()`).
    #[must_use]
    pub fn to_dec(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` ref.
    #[must_use]
    pub fn to_dec_ref(&self) -> &Decimal {
        &self.0
    }

    /// Converts the value to a 64-bit floating-point number.
    ///
    /// # Panics
    ///
    /// This method will panic if the conversion fails. Use `to_f64_checked()`
    /// or `to_f64_lossy()` for non-panicking alternatives.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.0
            .to_f64()
            .expect("Decimal to f64 conversion failed - value out of range")
    }

    /// Converts the value to f64, returning None if conversion fails.
    #[must_use]
    pub fn to_f64_checked(&self) -> Option<f64> {
        self.0.to_f64()
    }

    /// Converts the value to f64 with lossy conversion (returns 0.0 on failure).
    #[must_use]
    pub fn to_f64_lossy(&self) -> f64 {
        self.0.to_f64().unwrap_or(0.0)
    }

    /// Converts the value to a 64-bit signed integer.
    ///
    /// # Panics
    ///
    /// This method will panic if the conversion fails. Use `to_i64_checked()`
    /// for a non-panicking alternative.
    #[must_use]
    pub fn to_i64(&self) -> i64 {
        self.0
            .to_i64()
            .expect("Decimal to i64 conversion failed - value out of range")
    }

    /// Converts the value to i64, returning None if conversion fails.
    #[must_use]
    pub fn to_i64_checked(&self) -> Option<i64> {
        self.0.to_i64()
    }

    /// Converts the inner value to a `u64`.
    ///
    /// # Panics
    ///
    /// This method will panic if the conversion fails. Use `to_u64_checked()`
    /// for a non-panicking alternative.
    #[must_use]
    pub fn to_u64(&self) -> u64 {
        self.0
            .to_u64()
            .expect("Decimal to u64 conversion failed - value out of range")
    }

    /// Converts the value to u64, returning None if conversion fails.
    #[must_use]
    pub fn to_u64_checked(&self) -> Option<u64> {
        self.0.to_u64()
    }

    /// Converts the value to a usize.
    ///
    /// # Panics
    ///
    /// This method will panic if the conversion fails. Use `to_usize_checked()`
    /// for a non-panicking alternative.
    #[must_use]
    pub fn to_usize(&self) -> usize {
        self.0
            .to_usize()
            .expect("Decimal to usize conversion failed - value out of range")
    }

    /// Converts the value to usize, returning None if conversion fails.
    #[must_use]
    pub fn to_usize_checked(&self) -> Option<usize> {
        self.0.to_usize()
    }

    /// Returns the maximum of two `Positive` values.
    #[must_use]
    pub fn max(self, other: Positive) -> Positive {
        if self.0 > other.0 { self } else { other }
    }

    /// Returns the minimum of two `Positive` values.
    #[must_use]
    pub fn min(self, other: Positive) -> Positive {
        if self.0 < other.0 { self } else { other }
    }

    /// Rounds the value down to the nearest integer.
    #[must_use]
    pub fn floor(&self) -> Positive {
        Positive(self.0.floor())
    }

    /// Raises this value to an integer power.
    #[must_use]
    pub fn powi(&self, n: i64) -> Positive {
        Positive(self.0.powi(n))
    }

    /// Computes the result of raising the current value to the power of the given exponent.
    #[must_use]
    pub fn pow(&self, n: Positive) -> Positive {
        Positive(self.0.pow(n.to_dec()))
    }

    /// Raises the current value to the power of `n` using unsigned integer exponentiation.
    #[must_use]
    pub fn powu(&self, n: u64) -> Positive {
        Positive(self.0.powu(n))
    }

    /// Raises this value to a decimal power.
    #[must_use]
    pub fn powd(&self, p0: Decimal) -> Positive {
        Positive(self.0.powd(p0))
    }

    /// Rounds the value to the nearest integer.
    #[must_use]
    pub fn round(&self) -> Positive {
        Positive(self.0.round())
    }

    /// Rounds the current value to a "nice" number, based on its magnitude.
    #[must_use]
    pub fn round_to_nice_number(&self) -> Positive {
        let magnitude = self.log10().floor();
        let ten_pow = Positive::TEN.pow(magnitude);
        let normalized = self / &ten_pow;
        let nice_number = if normalized < dec!(1.5) {
            Positive::ONE
        } else if normalized < pos_or_panic!(3.0) {
            Positive::TWO
        } else if normalized < pos_or_panic!(7.0) {
            pos_or_panic!(5.0)
        } else {
            Positive::TEN
        };
        nice_number * pos_or_panic!(10.0).powu(magnitude.to_u64())
    }

    /// Calculates the square root of the value.
    ///
    /// # Panics
    ///
    /// This method will panic if the square root calculation fails.
    /// Use `sqrt_checked()` for a non-panicking alternative.
    #[must_use]
    pub fn sqrt(&self) -> Positive {
        Positive(self.0.sqrt().expect("Square root calculation failed"))
    }

    /// Calculates the square root, returning an error if it fails.
    pub fn sqrt_checked(&self) -> Result<Positive, PositiveError> {
        self.0.sqrt().map(Positive).ok_or_else(|| {
            PositiveError::arithmetic_error("sqrt", "square root calculation failed")
        })
    }

    /// Calculates the natural logarithm of the value.
    #[must_use]
    pub fn ln(&self) -> Positive {
        Positive(self.0.ln())
    }

    /// Rounds the value to a specified number of decimal places.
    #[must_use]
    pub fn round_to(&self, decimal_places: u32) -> Positive {
        Positive(self.0.round_dp(decimal_places))
    }

    /// Formats the value with a fixed number of decimal places.
    #[must_use]
    pub fn format_fixed_places(&self, decimal_places: u32) -> String {
        let rounded = self.round_to(decimal_places).to_f64();
        format!("{:.1$}", rounded, decimal_places as usize)
    }

    /// Calculates the exponential function e^x for this value.
    #[must_use]
    pub fn exp(&self) -> Positive {
        Positive(self.0.exp())
    }

    /// Clamps the value between a minimum and maximum.
    #[must_use]
    pub fn clamp(&self, min: Positive, max: Positive) -> Positive {
        if self < &min {
            min
        } else if self > &max {
            max
        } else {
            *self
        }
    }

    /// Checks if the value is exactly zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns the smallest integer greater than or equal to the value.
    #[must_use]
    pub fn ceiling(&self) -> Positive {
        Positive(self.to_dec().ceil())
    }

    /// Computes the base-10 logarithm of the value.
    #[must_use]
    pub fn log10(&self) -> Positive {
        Positive(self.0.log10())
    }

    /// Subtracts a decimal value, returning zero if the result would be negative.
    ///
    /// This method is not available when the `non-zero` feature is enabled
    /// because the result could be zero.
    #[cfg(not(feature = "non-zero"))]
    #[must_use]
    pub fn sub_or_zero(&self, other: &Decimal) -> Positive {
        if &self.0 > other {
            Positive(self.0 - other)
        } else {
            Positive(Decimal::ZERO)
        }
    }

    /// Subtracts a decimal value, returning None if the result would be negative.
    #[must_use]
    pub fn sub_or_none(&self, other: &Decimal) -> Option<Positive> {
        if &self.0 >= other {
            Some(Positive(self.0 - other))
        } else {
            None
        }
    }

    /// Checked subtraction that returns Result instead of panicking.
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Positive::new_decimal(self.0 - rhs.0)
    }

    /// Saturating subtraction that returns ZERO instead of negative.
    ///
    /// This method is not available when the `non-zero` feature is enabled
    /// because the result could be zero.
    #[cfg(not(feature = "non-zero"))]
    #[must_use]
    pub fn saturating_sub(&self, rhs: &Self) -> Self {
        if self.0 > rhs.0 {
            Positive(self.0 - rhs.0)
        } else {
            Positive::ZERO
        }
    }

    /// Checked division that returns Result instead of panicking.
    pub fn checked_div(&self, rhs: &Self) -> Result<Self, PositiveError> {
        if rhs.is_zero() {
            Err(PositiveError::arithmetic_error(
                "division",
                "division by zero",
            ))
        } else {
            Ok(Positive(self.0 / rhs.0))
        }
    }

    /// Checks whether the value is a multiple of another f64 value.
    #[must_use]
    pub fn is_multiple(&self, other: f64) -> bool {
        let value = self.to_f64();
        if !value.is_finite() {
            return false;
        }
        let remainder = value % other;
        remainder.abs() < f64::EPSILON || (other - remainder.abs()).abs() < f64::EPSILON
    }

    /// Checks whether the value is a multiple of another Positive value.
    #[must_use]
    pub fn is_multiple_of(&self, other: &Positive) -> bool {
        if other.is_zero() {
            return false;
        }
        let remainder = self.0 % other.0;
        remainder.abs() < EPSILON
    }

    /// Creates a new `Positive` value without checking if the value is non-negative.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value >= 0`. Using this with a negative value
    /// will violate the invariant of the `Positive` type and may cause undefined
    /// behavior in code that relies on the positivity guarantee.
    ///
    /// # Example
    ///
    /// ```rust
    /// use positive::Positive;
    /// use rust_decimal_macros::dec;
    ///
    /// // SAFETY: We know 5.0 is positive
    /// let value = unsafe { Positive::new_unchecked(dec!(5.0)) };
    /// assert_eq!(value.to_f64(), 5.0);
    /// ```
    #[must_use]
    pub const unsafe fn new_unchecked(value: Decimal) -> Self {
        Positive(value)
    }

    /// Crate-private const constructor used exclusively by `crate::constants`
    /// to define `Positive` constants in `const` context. The invariant is
    /// enforced by the callers: every constant in `crate::constants` is a
    /// strictly non-negative literal (or `>0` under the `non-zero` feature).
    #[must_use]
    pub(crate) const fn from_decimal_const(value: Decimal) -> Self {
        Positive(value)
    }
}

impl From<Positive> for Decimal {
    fn from(value: Positive) -> Self {
        value.0
    }
}

impl PartialEq<&Positive> for Positive {
    fn eq(&self, other: &&Positive) -> bool {
        self == *other
    }
}

impl From<Positive> for u64 {
    fn from(pos_u64: Positive) -> Self {
        pos_u64.0.to_u64().unwrap_or(0)
    }
}

impl From<&Positive> for f64 {
    fn from(value: &Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0)
    }
}

impl From<Positive> for f64 {
    fn from(value: Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0)
    }
}

impl From<Positive> for usize {
    fn from(value: Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0) as usize
    }
}

impl PartialEq<&Positive> for f64 {
    fn eq(&self, other: &&Positive) -> bool {
        self == &other.0.to_f64().unwrap_or(0.0)
    }
}

impl PartialOrd<&Positive> for f64 {
    fn partial_cmp(&self, other: &&Positive) -> Option<Ordering> {
        self.partial_cmp(&other.0.to_f64().unwrap_or(0.0))
    }
}

impl PartialEq<Positive> for f64 {
    fn eq(&self, other: &Positive) -> bool {
        self == &other.0.to_f64().unwrap_or(0.0)
    }
}

impl PartialOrd<Positive> for f64 {
    fn partial_cmp(&self, other: &Positive) -> Option<Ordering> {
        self.partial_cmp(&other.0.to_f64().unwrap_or(0.0))
    }
}

impl Mul<Positive> for f64 {
    type Output = f64;
    fn mul(self, rhs: Positive) -> Self::Output {
        self * rhs.to_f64()
    }
}

impl Div<Positive> for f64 {
    type Output = f64;
    fn div(self, rhs: Positive) -> Self::Output {
        self / rhs.to_f64()
    }
}

impl Sub<Positive> for f64 {
    type Output = f64;
    fn sub(self, rhs: Positive) -> Self::Output {
        self - rhs.to_f64()
    }
}

impl Add<Positive> for f64 {
    type Output = f64;
    fn add(self, rhs: Positive) -> Self::Output {
        self + rhs.to_f64()
    }
}

impl FromStr for Positive {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Decimal>() {
            Ok(value) if is_valid_positive_value(value) => Ok(Positive(value)),
            Ok(value) => Err(format!("Value must be positive, got {value}")),
            Err(e) => Err(format!("Failed to parse as Decimal: {e}")),
        }
    }
}

impl TryFrom<f64> for Positive {
    type Error = PositiveError;

    /// Attempts to convert an f64 to a Positive value.
    ///
    /// # Errors
    ///
    /// Returns `PositiveError` if the value is negative, NaN, or cannot be converted to Decimal.
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Positive::new(value)
    }
}

impl TryFrom<usize> for Positive {
    type Error = PositiveError;

    /// Attempts to convert a usize to a Positive value.
    ///
    /// # Errors
    ///
    /// Returns `PositiveError` if the value cannot be converted to Decimal.
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Positive::new(value as f64)
    }
}

impl TryFrom<Decimal> for Positive {
    type Error = PositiveError;

    /// Attempts to convert a Decimal to a Positive value.
    ///
    /// # Errors
    ///
    /// Returns `PositiveError` if the value is negative.
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Positive::new_decimal(value)
    }
}

impl TryFrom<&Decimal> for Positive {
    type Error = PositiveError;

    /// Attempts to convert a &Decimal to a Positive value.
    ///
    /// # Errors
    ///
    /// Returns `PositiveError` if the value is negative.
    fn try_from(value: &Decimal) -> Result<Self, Self::Error> {
        Positive::new_decimal(*value)
    }
}

impl TryFrom<i64> for Positive {
    type Error = PositiveError;

    /// Attempts to convert an i64 to a Positive value.
    ///
    /// # Errors
    ///
    /// Returns `PositiveError` if the value is negative.
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Positive::new_decimal(Decimal::from(value))
    }
}

impl TryFrom<u64> for Positive {
    type Error = PositiveError;

    /// Attempts to convert a u64 to a Positive value.
    ///
    /// # Errors
    ///
    /// This conversion is infallible for u64 since all values are non-negative.
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Positive::new_decimal(Decimal::from(value))
    }
}

impl From<&Positive> for Positive {
    fn from(value: &Positive) -> Self {
        Positive(value.0)
    }
}

impl Mul<f64> for Positive {
    type Output = Positive;
    fn mul(self, rhs: f64) -> Positive {
        Positive::new(self.to_f64() * rhs).expect("Multiplication result must be positive")
    }
}

impl Div<f64> for Positive {
    type Output = Positive;
    fn div(self, rhs: f64) -> Positive {
        Positive::new(self.to_f64() / rhs).expect("Division result must be positive")
    }
}

impl Div<f64> for &Positive {
    type Output = Positive;
    fn div(self, rhs: f64) -> Positive {
        Positive::new(self.to_f64() / rhs).expect("Division result must be positive")
    }
}

impl Sub<f64> for Positive {
    type Output = Positive;
    fn sub(self, rhs: f64) -> Self::Output {
        Positive::new(self.to_f64() - rhs).expect("Subtraction result must be positive")
    }
}

impl Add<f64> for Positive {
    type Output = Positive;
    fn add(self, rhs: f64) -> Self::Output {
        Positive::new(self.to_f64() + rhs).expect("Addition result must be positive")
    }
}

impl PartialOrd<f64> for Positive {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.to_f64().unwrap_or(0.0).partial_cmp(other)
    }
}

impl PartialEq<f64> for &Positive {
    fn eq(&self, other: &f64) -> bool {
        self.0.to_f64().unwrap_or(0.0) == *other
    }
}

impl PartialOrd<f64> for &Positive {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.to_f64().unwrap_or(0.0).partial_cmp(other)
    }
}

impl PartialEq<f64> for Positive {
    fn eq(&self, other: &f64) -> bool {
        self.to_f64() == *other
    }
}

impl Display for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Positive::INFINITY {
            write!(f, "{}", f64::MAX)
        } else if self.0.scale() == 0 {
            match self.0.to_i64() {
                Some(val) => write!(f, "{val}"),
                None => write!(f, "{}", self.0),
            }
        } else if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            let s = self.0.to_string();
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            write!(f, "{trimmed}")
        }
    }
}

impl fmt::Debug for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Positive::INFINITY {
            write!(f, "{}", f64::MAX)
        } else if self.0.scale() == 0 {
            match self.0.to_i64() {
                Some(val) => write!(f, "{val}"),
                None => write!(f, "{}", self.0),
            }
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl PartialEq<Decimal> for Positive {
    fn eq(&self, other: &Decimal) -> bool {
        (self.0 - *other).abs() <= EPSILON * Decimal::from(100)
    }
}

impl Serialize for Positive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self == Positive::INFINITY {
            return serializer.serialize_f64(f64::MAX);
        }
        if self.0.scale() == 0 {
            serializer.serialize_i64(
                self.0
                    .to_i64()
                    .ok_or_else(|| serde::ser::Error::custom("Failed to convert to i64"))?,
            )
        } else {
            serializer.serialize_f64(
                self.0
                    .to_f64()
                    .ok_or_else(|| serde::ser::Error::custom("Failed to convert to f64"))?,
            )
        }
    }
}

impl<'de> Deserialize<'de> for Positive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositiveVisitor;

        impl Visitor<'_> for PositiveVisitor {
            type Value = Positive;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a positive number")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Err(serde::de::Error::custom(format!(
                    "Invalid string: '{value}'. Expected a positive number."
                )))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decimal = Decimal::from(value);
                if !is_valid_positive_value(decimal) {
                    Err(serde::de::Error::custom("Expected a positive integer"))
                } else {
                    Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decimal = Decimal::from(value);
                if !is_valid_positive_value(decimal) {
                    Err(serde::de::Error::custom("Expected a positive integer"))
                } else {
                    Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.is_infinite() && value.is_sign_positive() {
                    return Ok(Positive::INFINITY);
                }
                if value == f64::MAX {
                    return Ok(Positive::INFINITY);
                }
                let decimal = Decimal::from_f64(value)
                    .ok_or_else(|| serde::de::Error::custom("Failed to convert f64 to Decimal"))?;
                if !is_valid_positive_value(decimal) {
                    Err(serde::de::Error::custom("Expected a positive float"))
                } else {
                    Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
                }
            }
        }

        deserializer.deserialize_any(PositiveVisitor)
    }
}

impl Add for Positive {
    type Output = Positive;
    fn add(self, other: Positive) -> Positive {
        Positive(self.0 + other.0)
    }
}

impl Sub for Positive {
    type Output = Positive;
    fn sub(self, rhs: Self) -> Self::Output {
        let result = self.0 - rhs.0;
        if result < Decimal::ZERO {
            panic!("Resulting value must be positive");
        } else {
            Positive(result)
        }
    }
}

impl Div for Positive {
    type Output = Positive;
    fn div(self, other: Positive) -> Self::Output {
        Positive(self.0 / other.0)
    }
}

impl Div for &Positive {
    type Output = Positive;
    fn div(self, other: &Positive) -> Self::Output {
        Positive(self.0 / other.0)
    }
}

impl Add<Decimal> for Positive {
    type Output = Positive;
    fn add(self, rhs: Decimal) -> Positive {
        Positive(self.0 + rhs)
    }
}

impl Add<&Decimal> for Positive {
    type Output = Positive;
    fn add(self, rhs: &Decimal) -> Self::Output {
        Positive::new_decimal(self.0 + rhs).expect("Addition result must be positive")
    }
}

impl Sub<Decimal> for Positive {
    type Output = Positive;
    fn sub(self, rhs: Decimal) -> Positive {
        Positive::new_decimal(self.0 - rhs).expect("Resulting value must be positive")
    }
}

impl Sub<&Decimal> for Positive {
    type Output = Positive;
    fn sub(self, rhs: &Decimal) -> Self::Output {
        Positive::new_decimal(self.0 - rhs).expect("Resulting value must be positive")
    }
}

impl AddAssign for Positive {
    fn add_assign(&mut self, other: Positive) {
        self.0 += other.0;
    }
}

impl AddAssign<Decimal> for Positive {
    fn add_assign(&mut self, rhs: Decimal) {
        self.0 += rhs;
    }
}

impl MulAssign<Decimal> for Positive {
    fn mul_assign(&mut self, rhs: Decimal) {
        self.0 *= rhs;
    }
}

impl Div<Decimal> for Positive {
    type Output = Positive;
    fn div(self, rhs: Decimal) -> Positive {
        Positive(self.0 / rhs)
    }
}

impl Div<&Decimal> for Positive {
    type Output = Positive;
    fn div(self, rhs: &Decimal) -> Self::Output {
        Positive::new_decimal(self.0 / rhs).expect("Division result must be positive")
    }
}

impl PartialOrd<Decimal> for Positive {
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Neg for Positive {
    type Output = Self;
    fn neg(self) -> Self::Output {
        panic!("Cannot negate a Positive value!");
    }
}

impl Mul for Positive {
    type Output = Positive;
    fn mul(self, other: Positive) -> Positive {
        Positive(self.0 * other.0)
    }
}

impl Mul<Decimal> for Positive {
    type Output = Positive;
    fn mul(self, rhs: Decimal) -> Positive {
        Positive(self.0 * rhs)
    }
}

impl Mul<Positive> for Decimal {
    type Output = Decimal;
    fn mul(self, rhs: Positive) -> Decimal {
        self * rhs.0
    }
}

impl Div<Positive> for Decimal {
    type Output = Decimal;
    fn div(self, rhs: Positive) -> Decimal {
        self / rhs.0
    }
}

impl Sub<Positive> for Decimal {
    type Output = Decimal;
    fn sub(self, rhs: Positive) -> Decimal {
        self - rhs.0
    }
}

impl Sub<&Positive> for Decimal {
    type Output = Decimal;
    fn sub(self, rhs: &Positive) -> Decimal {
        self - rhs.0
    }
}

impl Add<Positive> for Decimal {
    type Output = Decimal;
    fn add(self, rhs: Positive) -> Decimal {
        self + rhs.0
    }
}

impl Add<&Positive> for Decimal {
    type Output = Decimal;
    fn add(self, rhs: &Positive) -> Decimal {
        self + rhs.0
    }
}

impl std::ops::AddAssign<Positive> for Decimal {
    fn add_assign(&mut self, rhs: Positive) {
        *self += rhs.0;
    }
}

impl std::ops::AddAssign<&Positive> for Decimal {
    fn add_assign(&mut self, rhs: &Positive) {
        *self += rhs.0;
    }
}

impl std::ops::MulAssign<Positive> for Decimal {
    fn mul_assign(&mut self, rhs: Positive) {
        *self *= rhs.0;
    }
}

impl std::ops::MulAssign<&Positive> for Decimal {
    fn mul_assign(&mut self, rhs: &Positive) {
        *self *= rhs.0;
    }
}

impl PartialEq<Positive> for Decimal {
    fn eq(&self, other: &Positive) -> bool {
        *self == other.0
    }
}

impl From<&Positive> for Decimal {
    fn from(pos: &Positive) -> Self {
        pos.0
    }
}

#[cfg(not(feature = "non-zero"))]
impl Default for Positive {
    fn default() -> Self {
        Positive::ZERO
    }
}

#[cfg(feature = "non-zero")]
impl Default for Positive {
    fn default() -> Self {
        Positive::ONE
    }
}

impl AbsDiffEq for Positive {
    type Epsilon = Decimal;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        (self.0 - other.0).abs() <= epsilon
    }
}

impl RelativeEq for Positive {
    fn default_max_relative() -> Self::Epsilon {
        EPSILON * Decimal::from(100)
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        let abs_diff = (self.0 - other.0).abs();
        let largest = self.0.abs().max(other.0.abs());
        abs_diff <= epsilon || abs_diff <= max_relative * largest
    }
}

#[cfg(not(feature = "non-zero"))]
impl Sum for Positive {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new_decimal(sum).unwrap_or(Positive::ZERO)
    }
}

#[cfg(not(feature = "non-zero"))]
impl<'a> Sum<&'a Positive> for Positive {
    fn sum<I: Iterator<Item = &'a Positive>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new_decimal(sum).unwrap_or(Positive::ZERO)
    }
}
