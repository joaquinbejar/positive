/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Core implementation of the Positive type.

use crate::constants::{EPSILON, EPSILON_CMP};
use crate::error::PositiveError;
use approx::{AbsDiffEq, RelativeEq};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::cmp::{Ordering, PartialEq};
use std::fmt;
use std::fmt::Display;
#[cfg(not(feature = "non-zero"))]
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};
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

/// Returns the smallest value a `Positive` may hold under the active feature
/// configuration, as an exact `Decimal`.
///
/// Without the `non-zero` feature the minimum is `0`. With the `non-zero`
/// feature it is `1e-28`, the smallest strictly positive value
/// `rust_decimal::Decimal` can represent — not `f64::MIN_POSITIVE`, which is a
/// binary float bound with no bearing on `Decimal`'s range and which earlier
/// versions reported here incorrectly.
#[inline]
#[must_use]
pub(crate) fn min_bound() -> Decimal {
    #[cfg(feature = "non-zero")]
    {
        Decimal::new(1, 28)
    }
    #[cfg(not(feature = "non-zero"))]
    {
        Decimal::ZERO
    }
}

/// Returns the largest value a `Positive` may hold, as an exact `Decimal`.
///
/// This is `Decimal::MAX` under every feature configuration.
#[inline]
#[must_use]
pub(crate) fn max_bound() -> Decimal {
    Decimal::MAX
}

/// Determines if the given type parameter `T` is the `Positive` type.
#[must_use]
pub fn is_positive<T: 'static>() -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<Positive>()
}

/// Default rounding strategy used by every `Div` operator on `Positive`.
///
/// `Decimal` division is exact when the result fits in its 28-digit
/// mantissa, but when it does not the operation must round. We pick
/// banker's rounding (`MidpointNearestEven`) as the canonical strategy
/// because it is statistically unbiased and matches IEEE-754 default
/// behaviour for financial calculations. Callers needing a different
/// strategy should use [`Positive::checked_div_with_strategy`].
pub const DIV_ROUNDING_STRATEGY: RoundingStrategy = RoundingStrategy::MidpointNearestEven;

/// Maximum decimal places preserved by division results.
///
/// `rust_decimal::Decimal` carries up to 28 digits of precision; rounding
/// at that scale is effectively "keep full precision". This constant is
/// used by every `Div` operator and by
/// [`Positive::checked_div_with_strategy`].
pub(crate) const DIV_SCALE: u32 = 28;

/// Applies [`DIV_ROUNDING_STRATEGY`] to the result of a division.
///
/// Kept as a crate-private helper so every `Div` / `checked_div*`
/// operator on `Positive` rounds through the same point.
#[inline]
pub(crate) fn round_div(result: Decimal) -> Decimal {
    result.round_dp_with_strategy(DIV_SCALE, DIV_ROUNDING_STRATEGY)
}

// ===========================================================================
// Checked `Decimal` kernels
// ===========================================================================
//
// `rust_decimal`'s `+`, `-`, `*`, `/` and `%` operators panic on overflow and
// on division by zero. Any `Positive` method advertised as non-panicking must
// therefore never touch them: it has to go through `Decimal::checked_*` and
// map the resulting `None` to a typed error.
//
// These five kernels are the only place in the crate permitted to perform
// `Decimal` arithmetic. Every `checked_*` method on `Positive` delegates here,
// so overflow wording and division-by-zero handling cannot drift between call
// sites.

/// Adds two `Decimal`s, mapping overflow to a typed error.
#[inline]
pub(crate) fn dec_add(
    lhs: Decimal,
    rhs: Decimal,
    op: &'static str,
) -> Result<Decimal, PositiveError> {
    lhs.checked_add(rhs)
        .ok_or_else(|| PositiveError::arithmetic_error(op, "overflow"))
}

/// Subtracts two `Decimal`s, mapping overflow to a typed error.
#[inline]
pub(crate) fn dec_sub(
    lhs: Decimal,
    rhs: Decimal,
    op: &'static str,
) -> Result<Decimal, PositiveError> {
    lhs.checked_sub(rhs)
        .ok_or_else(|| PositiveError::arithmetic_error(op, "overflow"))
}

/// Multiplies two `Decimal`s, mapping overflow to a typed error.
#[inline]
pub(crate) fn dec_mul(
    lhs: Decimal,
    rhs: Decimal,
    op: &'static str,
) -> Result<Decimal, PositiveError> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| PositiveError::arithmetic_error(op, "overflow"))
}

/// Divides two `Decimal`s, mapping division by zero and overflow to typed
/// errors.
///
/// Division by zero is reported before the division is attempted, because
/// `Decimal::checked_div` and the `/` operator disagree on that case: the
/// operator panics.
#[inline]
pub(crate) fn dec_div(
    lhs: Decimal,
    rhs: Decimal,
    op: &'static str,
) -> Result<Decimal, PositiveError> {
    if rhs.is_zero() {
        return Err(PositiveError::arithmetic_error(op, "division by zero"));
    }
    lhs.checked_div(rhs)
        .ok_or_else(|| PositiveError::arithmetic_error(op, "overflow"))
}

// ===========================================================================
// Exact cross-type comparison
// ===========================================================================

/// Compares a `Positive`'s `Decimal` against an `f64` **exactly**.
///
/// The comparison lifts the `f64` into `Decimal` rather than lowering the
/// `Decimal` into `f64`. Lowering was the previous behaviour and it collapsed
/// distinct large decimal integers onto the same float — every value above
/// `2^53` compared equal to its neighbours — as well as mapping conversion
/// failures onto `0.0`, which made a huge value compare as zero.
///
/// The remaining cases are decided without conversion, because they have exact
/// answers that `Decimal` cannot represent:
///
/// - `NaN` is unordered against everything, so the result is `None`.
/// - `+inf` is greater than every `Positive`; `-inf` is smaller than every one.
/// - A finite `f64` whose magnitude exceeds `Decimal`'s range is likewise
///   larger (or smaller) than any representable value.
/// - A nonzero `f64` whose magnitude is below `Decimal`'s smallest step would
///   round to zero during conversion; its sign decides the answer instead of
///   the rounded value, so no nonzero float ever compares equal to zero.
#[inline]
fn cmp_decimal_f64(lhs: Decimal, rhs: f64) -> Option<Ordering> {
    if rhs.is_nan() {
        return None;
    }
    if rhs.is_infinite() {
        return Some(if rhs.is_sign_positive() {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    match Decimal::from_f64(rhs) {
        Some(rhs_dec) => {
            if rhs_dec.is_zero() && rhs != 0.0 {
                // The conversion underflowed a nonzero float to zero.
                return Some(if rhs > 0.0 {
                    // A tiny positive float sits strictly between zero and
                    // Decimal's smallest positive value.
                    if lhs.is_zero() {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                } else {
                    // Every `Positive` is at least zero, so it exceeds any
                    // negative float.
                    Ordering::Greater
                });
            }
            lhs.partial_cmp(&rhs_dec)
        }
        // Finite, but outside `Decimal`'s range: its sign decides the answer.
        None => Some(if rhs > 0.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }),
    }
}

/// Computes the remainder of two `Decimal`s, mapping division by zero and
/// overflow to typed errors.
#[inline]
pub(crate) fn dec_rem(
    lhs: Decimal,
    rhs: Decimal,
    op: &'static str,
) -> Result<Decimal, PositiveError> {
    if rhs.is_zero() {
        return Err(PositiveError::arithmetic_error(op, "division by zero"));
    }
    lhs.checked_rem(rhs)
        .ok_or_else(|| PositiveError::arithmetic_error(op, "overflow"))
}

/// Panics with a uniform message when a `Positive` arithmetic operation
/// overflows the underlying `Decimal` range.
///
/// Marked `#[cold]` and `#[inline(never)]` so the happy path stays lean.
#[cold]
#[inline(never)]
pub(crate) fn overflow_panic(op: &'static str) -> ! {
    panic!("Positive arithmetic overflow in {op}")
}

/// Panics with a uniform message when the result of a `Positive`
/// arithmetic operation would violate the positivity invariant
/// (negative, or zero under the `non-zero` feature).
///
/// Marked `#[cold]` and `#[inline(never)]` so the happy path stays lean.
#[cold]
#[inline(never)]
pub(crate) fn invariant_panic(op: &'static str) -> ! {
    panic!("Positive invariant broken in {op}: result would be non-positive")
}

/// Panics with a uniform message when a mathematical operation is applied
/// outside its domain — the logarithm of zero, for instance.
///
/// Kept distinct from [`overflow_panic`] and [`invariant_panic`] because the
/// three failures are genuinely different: the input was invalid, the result
/// did not fit, or the result was not positive.
#[cold]
#[inline(never)]
pub(crate) fn domain_panic(op: &'static str) -> ! {
    panic!("Positive domain error in {op}: value is outside the operation's domain")
}

/// Validates a caller-supplied number of decimal places against the range
/// `rust_decimal::Decimal` actually supports.
///
/// This runs **before** any rounding or allocation. `format_fixed_places` used
/// to pass its argument straight to `format!` as the precision, so a value
/// like `u32::MAX` asked for a four-billion-character `String` and aborted the
/// process on allocation failure. Validating first turns that into a typed
/// error.
#[inline]
pub(crate) fn validate_precision(decimal_places: u32) -> Result<u32, PositiveError> {
    if decimal_places > Decimal::MAX_SCALE {
        return Err(PositiveError::invalid_precision(
            decimal_places,
            "decimal supports at most 28 decimal places",
        ));
    }
    Ok(decimal_places)
}

/// Panics with a uniform message when a clamp range is inverted.
#[cold]
#[inline(never)]
pub(crate) fn inverted_range_panic() -> ! {
    panic!("Positive clamp range is inverted: min is greater than max")
}

/// Panics with a uniform message when a caller-supplied precision is outside
/// the range `Decimal` supports.
#[cold]
#[inline(never)]
pub(crate) fn precision_panic(decimal_places: u32) -> ! {
    panic!(
        "Positive precision {decimal_places} is invalid: decimal supports at most {} decimal places",
        Decimal::MAX_SCALE
    )
}

/// Panics with a uniform message when a value cannot be represented in the
/// destination primitive type.
#[cold]
#[inline(never)]
pub(crate) fn conversion_panic(target: &'static str) -> ! {
    panic!("Positive conversion to {target} failed: value is out of range")
}

/// Converts a checked result into the panic documented on the non-checked
/// wrapper, preserving the distinction between an overflow and an invariant
/// violation.
///
/// Every panicking method and operator on `Positive` is a thin wrapper over
/// its `checked_*` counterpart through this function, so the two can never
/// disagree about what succeeds. It is the single point at which a typed
/// error becomes a panic.
#[inline]
pub(crate) fn unwrap_or_panic(
    result: Result<Positive, PositiveError>,
    op: &'static str,
) -> Positive {
    match result {
        Ok(value) => value,
        Err(PositiveError::OutOfBounds { .. }) => invariant_panic(op),
        Err(_) => overflow_panic(op),
    }
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
    /// The largest value a `Positive` can hold: `Decimal::MAX`
    /// (79,228,162,514,264,337,593,543,950,335).
    ///
    /// Mirrors [`crate::constants::MAX`]. This is a real maximum, not an
    /// infinity — `Decimal` has no infinite value — and every operation treats
    /// it as the finite number it is.
    pub const MAX: Positive = crate::constants::MAX;

    /// Represents the maximum positive value possible (effectively infinity).
    ///
    /// # Deprecated
    ///
    /// Use [`Positive::MAX`]. Same value, accurate name; see the constant's
    /// documentation for why the old one was misleading.
    #[deprecated(
        since = "0.6.0",
        note = "renamed to `Positive::MAX`: the value is Decimal::MAX, not an infinity"
    )]
    #[allow(deprecated)]
    pub const INFINITY: Positive = crate::constants::INFINITY;

    /// Number of days in a year, in days (365, ignoring leap years).
    ///
    /// Mirrors [`crate::constants::DAYS_IN_A_YEAR`]. Use this for day-count
    /// conventions that assume a fixed 365-day year, such as ACT/365.
    pub const DAYS_IN_A_YEAR: Positive = crate::constants::DAYS_IN_A_YEAR;

    /// Creates a new `Positive` value from a 64-bit floating-point number.
    ///
    /// Without the `non-zero` feature, values >= 0 are accepted.
    /// With the `non-zero` feature, only values > 0 are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidValue`] when `value` is `NaN` or
    /// infinite, [`PositiveError::ConversionError`] when it is finite but
    /// outside the range `Decimal` can represent, and
    /// [`PositiveError::OutOfBounds`] when it converts cleanly but breaks the
    /// positivity invariant. The `OutOfBounds` bounds are exact `Decimal`
    /// values reflecting the active feature configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError};
    ///
    /// assert!(Positive::new(1.5).is_ok());
    /// assert!(matches!(
    ///     Positive::new(-1.0).unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// assert!(matches!(
    ///     Positive::new(f64::NAN).unwrap_err(),
    ///     PositiveError::InvalidValue { .. }
    /// ));
    /// ```
    #[must_use = "constructor returns a Result; ignoring the Positive discards a validated invariant"]
    pub fn new(value: f64) -> Result<Self, PositiveError> {
        if value.is_nan() {
            return Err(PositiveError::invalid_value("NaN", "value is not a number"));
        }
        if value.is_infinite() {
            return Err(PositiveError::invalid_value(
                &value.to_string(),
                "value is infinite and has no decimal representation",
            ));
        }
        match Decimal::from_f64(value) {
            Some(dec) if is_valid_positive_value(dec) => Ok(Positive(dec)),
            Some(dec) => Err(PositiveError::out_of_bounds(dec, min_bound(), max_bound())),
            None => Err(PositiveError::conversion_error(
                "f64",
                "Positive",
                "value is outside the range representable as a decimal",
            )),
        }
    }

    /// Creates a new `Positive` value directly from a `Decimal`.
    ///
    /// Without the `non-zero` feature, values >= 0 are accepted.
    /// With the `non-zero` feature, only values > 0 are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` breaks the
    /// positivity invariant. The reported value and both bounds are exact
    /// `Decimal`s, so no precision is lost in the diagnostic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError};
    /// use rust_decimal_macros::dec;
    ///
    /// assert!(Positive::new_decimal(dec!(1.5)).is_ok());
    /// assert!(matches!(
    ///     Positive::new_decimal(dec!(-1)).unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// ```
    #[must_use = "constructor returns a Result; ignoring the Positive discards a validated invariant"]
    pub fn new_decimal(value: Decimal) -> Result<Self, PositiveError> {
        if is_valid_positive_value(value) {
            Ok(Positive(value))
        } else {
            Err(PositiveError::out_of_bounds(
                value,
                min_bound(),
                max_bound(),
            ))
        }
    }

    /// Returns the inner `Decimal` value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` value (alias for `value()`).
    #[inline]
    #[must_use]
    pub fn to_dec(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` ref.
    #[inline]
    #[must_use]
    pub fn to_dec_ref(&self) -> &Decimal {
        &self.0
    }

    /// Converts the value to a 64-bit floating-point number.
    ///
    /// This conversion is **infallible**. `rust_decimal` implements
    /// `Decimal::to_f64` as `Some(self.as_f64())`, and `as_f64` always
    /// produces a value; calling `as_f64` directly encodes that fact in the
    /// signature instead of asserting it at runtime with an `expect`, which is
    /// what earlier versions did.
    ///
    /// It is still **lossy**: `f64` carries about 15 significant digits, so
    /// magnitudes beyond `2^53` and fractions beyond that precision are
    /// rounded. Use [`Positive::to_dec`] when precision matters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(2.5).to_f64(), 2.5);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.0.as_f64()
    }

    /// Converts the value to f64, returning None if conversion fails.
    ///
    /// Retained for source compatibility; it always returns `Some`, because
    /// the conversion cannot fail. Prefer [`Positive::to_f64`].
    #[inline]
    #[must_use]
    pub fn to_f64_checked(&self) -> Option<f64> {
        Some(self.to_f64())
    }

    /// Converts the value to f64 with lossy conversion (returns 0.0 on failure).
    ///
    /// # Deprecated
    ///
    /// There is no failure case to fall back from: [`Positive::to_f64`] is
    /// infallible and returns the same value.
    #[deprecated(
        since = "0.6.0",
        note = "`to_f64` is infallible and returns the same value; this alias will be removed after 0.6.0"
    )]
    #[inline]
    #[must_use]
    pub fn to_f64_lossy(&self) -> f64 {
        self.to_f64()
    }

    /// Converts the value to a 64-bit signed integer, truncating any fraction.
    ///
    /// # Deprecated
    ///
    /// This method panics for values that are perfectly valid `Positive`s but
    /// exceed `i64::MAX`. Use `i64::try_from(positive)` for a typed error, or
    /// [`Positive::to_i64_checked`] for an `Option`.
    ///
    /// # Panics
    ///
    /// Panics when the truncated value does not fit in an `i64`.
    #[deprecated(
        since = "0.6.0",
        note = "panics for valid values above i64::MAX; use `i64::try_from(value)` or `to_i64_checked`"
    )]
    #[must_use]
    pub fn to_i64(&self) -> i64 {
        match self.0.to_i64() {
            Some(value) => value,
            None => conversion_panic("i64"),
        }
    }

    /// Converts the value to i64, returning None if it does not fit.
    ///
    /// Any fraction is truncated toward zero.
    #[inline]
    #[must_use]
    pub fn to_i64_checked(&self) -> Option<i64> {
        self.0.to_i64()
    }

    /// Converts the inner value to a `u64`, truncating any fraction.
    ///
    /// # Deprecated
    ///
    /// This method panics for values that are perfectly valid `Positive`s but
    /// exceed `u64::MAX`. Use `u64::try_from(positive)` for a typed error, or
    /// [`Positive::to_u64_checked`] for an `Option`.
    ///
    /// # Panics
    ///
    /// Panics when the truncated value does not fit in a `u64`.
    #[deprecated(
        since = "0.6.0",
        note = "panics for valid values above u64::MAX; use `u64::try_from(value)` or `to_u64_checked`"
    )]
    #[must_use]
    pub fn to_u64(&self) -> u64 {
        match self.0.to_u64() {
            Some(value) => value,
            None => conversion_panic("u64"),
        }
    }

    /// Converts the value to u64, returning None if it does not fit.
    ///
    /// Any fraction is truncated toward zero.
    #[inline]
    #[must_use]
    pub fn to_u64_checked(&self) -> Option<u64> {
        self.0.to_u64()
    }

    /// Converts the value to a usize, truncating any fraction.
    ///
    /// # Deprecated
    ///
    /// This method panics for values that are perfectly valid `Positive`s but
    /// exceed `usize::MAX`. Use `usize::try_from(positive)` for a typed error,
    /// or [`Positive::to_usize_checked`] for an `Option`.
    ///
    /// # Panics
    ///
    /// Panics when the truncated value does not fit in a `usize`.
    #[deprecated(
        since = "0.6.0",
        note = "panics for valid values above usize::MAX; use `usize::try_from(value)` or `to_usize_checked`"
    )]
    #[must_use]
    pub fn to_usize(&self) -> usize {
        match self.0.to_usize() {
            Some(value) => value,
            None => conversion_panic("usize"),
        }
    }

    /// Converts the value to usize, returning None if it does not fit.
    ///
    /// Any fraction is truncated toward zero.
    #[inline]
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
    ///
    /// # Panics
    ///
    /// Panics when the floored result would break the positivity invariant.
    /// Under the `non-zero` feature this includes every value below one, whose
    /// floor is zero — for example `0.5`. Without that feature this method
    /// cannot panic. Use [`Positive::checked_floor`] for the non-panicking
    /// form.
    #[must_use]
    pub fn floor(&self) -> Positive {
        unwrap_or_panic(self.checked_floor(), "floor")
    }

    /// Rounds the value down to the nearest integer, returning an error
    /// instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the floored result would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(1.9).checked_floor().unwrap(), pos_or_panic!(1.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_floor(&self) -> Result<Positive, PositiveError> {
        Positive::new_decimal(self.0.floor())
    }

    /// Raises this value to an integer power.
    ///
    /// # Panics
    ///
    /// Panics when the power cannot be computed — for example a zero base with
    /// a negative exponent — or when the result would break the positivity
    /// invariant. Use [`Positive::checked_powi`] for the non-panicking form.
    #[must_use]
    pub fn powi(&self, n: i64) -> Positive {
        unwrap_or_panic(self.checked_powi(n), "powi")
    }

    /// Raises this value to an integer power, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power is outside
    /// the domain — a zero base with a negative exponent — or overflows, and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(2.0).checked_powi(3).unwrap(), pos_or_panic!(8.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the domain error"]
    pub fn checked_powi(&self, n: i64) -> Result<Positive, PositiveError> {
        let result = self.0.checked_powi(n).ok_or_else(|| {
            PositiveError::arithmetic_error("powi", "power is undefined or overflows")
        })?;
        Positive::new_decimal(result)
    }

    /// Computes the result of raising the current value to the power of the given exponent.
    ///
    /// # Panics
    ///
    /// Panics when the power cannot be computed or the result would break the
    /// positivity invariant. Use [`Positive::checked_pow`] for the
    /// non-panicking form.
    #[must_use]
    pub fn pow(&self, n: Positive) -> Positive {
        unwrap_or_panic(self.checked_pow(n), "pow")
    }

    /// Raises this value to a `Positive` power, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power is outside
    /// the domain or overflows, and [`PositiveError::OutOfBounds`] when the
    /// result would break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// let value = pos_or_panic!(2.0);
    /// assert_eq!(value.checked_pow(pos_or_panic!(3.0)).unwrap(), pos_or_panic!(8.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the domain error"]
    pub fn checked_pow(&self, n: Positive) -> Result<Positive, PositiveError> {
        self.checked_powd(n.to_dec())
    }

    /// Raises the current value to the power of `n` using unsigned integer exponentiation.
    ///
    /// # Panics
    ///
    /// Panics when the power overflows or the result would break the
    /// positivity invariant. Use [`Positive::checked_powu`] for the
    /// non-panicking form.
    #[must_use]
    pub fn powu(&self, n: u64) -> Positive {
        unwrap_or_panic(self.checked_powu(n), "powu")
    }

    /// Raises this value to an unsigned integer power, returning an error
    /// instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power overflows and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant — under the `non-zero` feature, when it underflows
    /// to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(2.0).checked_powu(3).unwrap(), pos_or_panic!(8.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_powu(&self, n: u64) -> Result<Positive, PositiveError> {
        let result = self
            .0
            .checked_powu(n)
            .ok_or_else(|| PositiveError::arithmetic_error("powu", "power overflows"))?;
        Positive::new_decimal(result)
    }

    /// Raises this value to a decimal power.
    ///
    /// # Panics
    ///
    /// Panics when the power cannot be computed or the result would break the
    /// positivity invariant. Use [`Positive::checked_powd`] for the
    /// non-panicking form.
    #[must_use]
    pub fn powd(&self, p0: Decimal) -> Positive {
        unwrap_or_panic(self.checked_powd(p0), "powd")
    }

    /// Raises this value to a decimal power, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power is outside
    /// the domain or overflows, and [`PositiveError::OutOfBounds`] when the
    /// result would break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(pos_or_panic!(2.0).checked_powd(dec!(3)).unwrap(), pos_or_panic!(8.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the domain error"]
    pub fn checked_powd(&self, p0: Decimal) -> Result<Positive, PositiveError> {
        if self.0.is_zero() && p0.is_sign_negative() {
            return Err(PositiveError::arithmetic_error(
                "powd",
                "zero to a negative power is undefined",
            ));
        }
        let result = self.0.checked_powd(p0).ok_or_else(|| {
            PositiveError::arithmetic_error("powd", "power is undefined or overflows")
        })?;
        Positive::new_decimal(result)
    }

    /// Rounds the value to the nearest integer.
    ///
    /// # Panics
    ///
    /// Panics when the rounded result would break the positivity invariant.
    /// Under the `non-zero` feature this includes every value below `0.5`,
    /// which rounds to zero. Without that feature this method cannot panic.
    /// Use [`Positive::checked_round`] for the non-panicking form.
    #[must_use]
    pub fn round(&self) -> Positive {
        unwrap_or_panic(self.checked_round(), "round")
    }

    /// Rounds the value to the nearest integer, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the rounded result would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(1.6).checked_round().unwrap(), pos_or_panic!(2.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_round(&self) -> Result<Positive, PositiveError> {
        Positive::new_decimal(self.0.round())
    }

    /// Rounds the current value to a "nice" number, based on its magnitude.
    ///
    /// The magnitude is computed entirely in `Decimal`. Routing it through
    /// `Positive` — as earlier versions did — meant the intermediate magnitude
    /// was zero for every input below ten, which is invalid under the
    /// `non-zero` feature, and that the final scaling went through
    /// `magnitude.to_u64()`, which cannot represent the negative magnitude of
    /// an input below one.
    ///
    /// A zero input maps to zero: it is already the nicest number at its own
    /// magnitude, and `log10(0)` is undefined so there is nothing else to
    /// compute. Under the `non-zero` feature zero is not constructible, so
    /// that case cannot arise.
    ///
    /// # Panics
    ///
    /// Panics when the scaled result overflows `Decimal` or breaks the
    /// positivity invariant. Use [`Positive::checked_round_to_nice_number`]
    /// for the non-panicking form.
    #[must_use]
    pub fn round_to_nice_number(&self) -> Positive {
        unwrap_or_panic(self.checked_round_to_nice_number(), "round_to_nice_number")
    }

    /// Rounds to a "nice" number, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the magnitude cannot be
    /// computed or the scaled result overflows, and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(
    ///     pos_or_panic!(4.0).checked_round_to_nice_number().unwrap(),
    ///     pos_or_panic!(5.0)
    /// );
    /// assert_eq!(
    ///     pos_or_panic!(0.12).checked_round_to_nice_number().unwrap(),
    ///     pos_or_panic!(0.1)
    /// );
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the domain error"]
    pub fn checked_round_to_nice_number(&self) -> Result<Positive, PositiveError> {
        if self.0.is_zero() {
            return Positive::new_decimal(Decimal::ZERO);
        }
        let magnitude = self
            .0
            .checked_log10()
            .ok_or_else(|| {
                PositiveError::arithmetic_error(
                    "round_to_nice_number",
                    "base-10 logarithm is undefined for this value",
                )
            })?
            .floor();
        let ten_pow = Decimal::TEN.checked_powd(magnitude).ok_or_else(|| {
            PositiveError::arithmetic_error("round_to_nice_number", "magnitude overflows")
        })?;
        let normalized = dec_div(self.0, ten_pow, "round_to_nice_number")?;
        let nice_number = if normalized < dec!(1.5) {
            Decimal::ONE
        } else if normalized < dec!(3.0) {
            dec!(2)
        } else if normalized < dec!(7.0) {
            dec!(5)
        } else {
            dec!(10)
        };
        Positive::new_decimal(dec_mul(nice_number, ten_pow, "round_to_nice_number")?)
    }

    /// Calculates the square root of the value.
    ///
    /// # Panics
    ///
    /// Panics when the square root cannot be computed, or when the result
    /// would break the positivity invariant. Use [`Positive::checked_sqrt`]
    /// for the non-panicking form.
    #[must_use]
    pub fn sqrt(&self) -> Positive {
        unwrap_or_panic(self.checked_sqrt(), "sqrt")
    }

    /// Calculates the square root, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the square root cannot
    /// be computed, and [`PositiveError::OutOfBounds`] when the result would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(16.0).checked_sqrt().unwrap(), pos_or_panic!(4.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_sqrt(&self) -> Result<Positive, PositiveError> {
        let root = self.0.sqrt().ok_or_else(|| {
            PositiveError::arithmetic_error("sqrt", "square root calculation failed")
        })?;
        Positive::new_decimal(root)
    }

    /// Calculates the square root, returning an error if it fails.
    ///
    /// # Errors
    ///
    /// See [`Positive::checked_sqrt`].
    #[deprecated(
        since = "0.6.0",
        note = "renamed to `checked_sqrt` for consistency with the rest of the checked API"
    )]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn sqrt_checked(&self) -> Result<Positive, PositiveError> {
        self.checked_sqrt()
    }

    /// Calculates the natural logarithm of the value.
    ///
    /// Returns a [`Decimal`], not a `Positive`: the logarithm of a positive
    /// number is not itself necessarily positive. `ln(0.5)` is `-0.693…`, and
    /// earlier versions returned that inside a `Positive`, silently breaking
    /// the type's central guarantee.
    ///
    /// # Panics
    ///
    /// Panics for a zero input, for which the natural logarithm is undefined.
    /// Zero is not constructible under the `non-zero` feature, so this cannot
    /// happen there. Use [`Positive::checked_ln`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal::Decimal;
    ///
    /// let half = pos_or_panic!(0.5);
    /// assert!(half.ln() < Decimal::ZERO);
    /// ```
    #[inline]
    #[must_use]
    pub fn ln(&self) -> Decimal {
        match self.checked_ln() {
            Ok(value) => value,
            Err(_) => domain_panic("ln"),
        }
    }

    /// Calculates the natural logarithm, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the logarithm is
    /// undefined for the value — that is, for zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError};
    ///
    /// assert!(Positive::ONE.checked_ln().unwrap().is_zero());
    /// ```
    #[inline]
    #[must_use = "checked mathematics returns a Result; ignoring it silences the domain error"]
    pub fn checked_ln(&self) -> Result<Decimal, PositiveError> {
        self.0.checked_ln().ok_or_else(|| {
            PositiveError::arithmetic_error("ln", "natural logarithm is undefined for zero")
        })
    }

    /// Computes the base-10 logarithm of the value.
    ///
    /// Returns a [`Decimal`], not a `Positive`, for the same reason as
    /// [`Positive::ln`]: `log10(0.5)` is `-0.301…`.
    ///
    /// # Panics
    ///
    /// Panics for a zero input, for which the logarithm is undefined. Use
    /// [`Positive::checked_log10`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(pos_or_panic!(100.0).log10(), dec!(2));
    /// ```
    #[inline]
    #[must_use]
    pub fn log10(&self) -> Decimal {
        match self.checked_log10() {
            Ok(value) => value,
            Err(_) => domain_panic("log10"),
        }
    }

    /// Computes the base-10 logarithm, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the logarithm is
    /// undefined for the value — that is, for zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(pos_or_panic!(100.0).checked_log10().unwrap(), dec!(2));
    /// ```
    #[inline]
    #[must_use = "checked mathematics returns a Result; ignoring it silences the domain error"]
    pub fn checked_log10(&self) -> Result<Decimal, PositiveError> {
        self.0.checked_log10().ok_or_else(|| {
            PositiveError::arithmetic_error("log10", "base-10 logarithm is undefined for zero")
        })
    }

    /// Rounds the value to a specified number of decimal places.
    ///
    /// # Panics
    ///
    /// Panics when the rounded result would break the positivity invariant.
    /// Under the `non-zero` feature this includes any value that rounds to
    /// zero at the requested scale — for example `0.5` at `round_to(0)`.
    /// Without that feature this method cannot panic. Use
    /// [`Positive::checked_round_to`] for the non-panicking form.
    #[inline]
    #[must_use]
    pub fn round_to(&self, decimal_places: u32) -> Positive {
        match validate_precision(decimal_places) {
            Ok(_) => unwrap_or_panic(self.checked_round_to(decimal_places), "round_to"),
            Err(_) => precision_panic(decimal_places),
        }
    }

    /// Rounds to a specified number of decimal places, returning an error
    /// instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the rounded result would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(
    ///     pos_or_panic!(1.2345).checked_round_to(2).unwrap(),
    ///     pos_or_panic!(1.23)
    /// );
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_round_to(&self, decimal_places: u32) -> Result<Positive, PositiveError> {
        let decimal_places = validate_precision(decimal_places)?;
        Positive::new_decimal(self.0.round_dp(decimal_places))
    }

    /// Formats the value with a fixed number of decimal places.
    ///
    /// Rounds the underlying `Decimal` at `decimal_places` using its
    /// default rounding strategy and formats the result. No
    /// `f64` round-trip, so precision is preserved beyond the ~15 digits
    /// of `f64`.
    ///
    /// # Allocation
    ///
    /// The returned `String` is at most a few dozen bytes, because
    /// `decimal_places` is validated against `Decimal::MAX_SCALE` (28) before
    /// any allocation happens. Earlier versions passed the argument straight
    /// through to `format!`, so `format_fixed_places(u32::MAX)` asked for a
    /// four-billion-character string and aborted the process on allocation
    /// failure — a denial of service reachable from user or config input.
    ///
    /// # Panics
    ///
    /// Panics when `decimal_places` exceeds 28. Use
    /// [`Positive::checked_format_fixed_places`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(1.2345).format_fixed_places(2), "1.23");
    /// ```
    #[inline]
    #[must_use]
    pub fn format_fixed_places(&self, decimal_places: u32) -> String {
        match self.checked_format_fixed_places(decimal_places) {
            Ok(formatted) => formatted,
            Err(_) => precision_panic(decimal_places),
        }
    }

    /// Formats the value with a fixed number of decimal places, returning an
    /// error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidPrecision`] when `decimal_places`
    /// exceeds the 28 places `Decimal` supports. The check runs before any
    /// rounding or allocation, so an absurd precision costs nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    ///
    /// let value = pos_or_panic!(1.2345);
    /// assert_eq!(value.checked_format_fixed_places(2).unwrap(), "1.23");
    /// assert!(matches!(
    ///     value.checked_format_fixed_places(u32::MAX).unwrap_err(),
    ///     PositiveError::InvalidPrecision { .. }
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked formatting returns a Result; ignoring it silences the precision error"]
    pub fn checked_format_fixed_places(
        &self,
        decimal_places: u32,
    ) -> Result<String, PositiveError> {
        let decimal_places = validate_precision(decimal_places)?;
        let rounded = self.0.round_dp(decimal_places);

        // The obvious implementation — `format!("{:.n$}", rounded, n)` — routes
        // through `Decimal`'s own formatter, which writes into a fixed-capacity
        // buffer sized for a decimal's normal width. Asking for 28 places on a
        // 29-digit value needs 58 characters and overflows it, panicking with
        // `CapacityError` inside the dependency: `Positive::MAX` at 28 places
        // aborted, even though both the value and the precision are valid.
        //
        // Padding the exact representation ourselves has no such limit, and
        // produces identical output for every input the old path survived.
        let mut text = rounded.to_string();
        if decimal_places == 0 {
            return Ok(text);
        }

        let fractional_len = match text.find('.') {
            Some(point) => text.len() - point - 1,
            None => {
                text.push('.');
                0
            }
        };
        for _ in fractional_len..decimal_places as usize {
            text.push('0');
        }
        Ok(text)
    }

    /// Calculates the exponential function e^x for this value.
    ///
    /// The result is always at least one, so it can never break the positivity
    /// invariant; only overflow can fail.
    ///
    /// # Panics
    ///
    /// Panics when the result overflows `Decimal`. Use
    /// [`Positive::checked_exp`] for the non-panicking form.
    #[inline]
    #[must_use]
    pub fn exp(&self) -> Positive {
        unwrap_or_panic(self.checked_exp(), "exp")
    }

    /// Calculates e^x, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the result overflows
    /// `Decimal`, which happens for inputs well below one hundred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError, pos_or_panic};
    ///
    /// assert!(Positive::ONE.checked_exp().is_ok());
    /// assert!(matches!(
    ///     pos_or_panic!(1000.0).checked_exp().unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked mathematics returns a Result; ignoring it silences the overflow error"]
    pub fn checked_exp(&self) -> Result<Positive, PositiveError> {
        let result = self
            .0
            .checked_exp()
            .ok_or_else(|| PositiveError::arithmetic_error("exp", "result overflows"))?;
        Positive::new_decimal(result)
    }

    /// Clamps the value between a minimum and maximum.
    ///
    /// Takes `self` by value. With a `&self` receiver — as earlier versions
    /// had — Rust's method resolution preferred `Ord::clamp` for any owned
    /// `Positive`, because that one needs no autoref. The crate's own clamp
    /// was therefore only reachable through a reference, and the two disagreed
    /// on inverted ranges: `Ord::clamp` asserted, this one silently returned a
    /// bound. Taking `self` makes the inherent method win uniformly, so there
    /// is one contract regardless of how the receiver is written.
    ///
    /// # Panics
    ///
    /// Panics when `min > max`. The standard `clamp` contract treats an
    /// inverted range as a caller bug, and this method follows it. The
    /// reference path used to return whichever bound the if/else chain reached
    /// first, so the same impossible interval produced `min` for a low input
    /// and `max` for a high one, with nothing to tell the caller.
    ///
    /// Use [`Positive::checked_clamp`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// let value = pos_or_panic!(7.0);
    /// assert_eq!(value.clamp(pos_or_panic!(1.0), pos_or_panic!(5.0)), pos_or_panic!(5.0));
    /// assert_eq!(value.clamp(pos_or_panic!(8.0), pos_or_panic!(10.0)), pos_or_panic!(8.0));
    /// assert_eq!(value.clamp(pos_or_panic!(1.0), pos_or_panic!(10.0)), value);
    /// ```
    #[must_use]
    pub fn clamp(self, min: Positive, max: Positive) -> Positive {
        match self.checked_clamp(min, max) {
            Ok(clamped) => clamped,
            Err(_) => inverted_range_panic(),
        }
    }

    /// Clamps the value between a minimum and maximum, returning an error
    /// instead of panicking on an inverted range.
    ///
    /// Equal bounds are valid and collapse the interval to a single value.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `min > max`. The error
    /// carries the offending bounds as exact `Decimal`s: `value` is the
    /// requested `min`, with `min`/`max` describing the range it had to fall
    /// within to be a valid lower bound.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    ///
    /// let value = pos_or_panic!(7.0);
    /// assert_eq!(
    ///     value.checked_clamp(pos_or_panic!(1.0), pos_or_panic!(5.0)).unwrap(),
    ///     pos_or_panic!(5.0)
    /// );
    /// assert!(matches!(
    ///     value.checked_clamp(pos_or_panic!(10.0), pos_or_panic!(1.0)).unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// ```
    #[must_use = "checked clamping returns a Result; ignoring it silences the inverted-range error"]
    pub fn checked_clamp(self, min: Positive, max: Positive) -> Result<Positive, PositiveError> {
        if min > max {
            return Err(PositiveError::out_of_bounds(min.0, min_bound(), max.0));
        }
        Ok(if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        })
    }

    /// Checks if the value is exactly zero.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Compares this value with a `Decimal` within an absolute tolerance.
    ///
    /// `==` against a `Decimal` is exact. This is the explicitly named
    /// approximate comparison that replaces the epsilon behaviour `==` used to
    /// have implicitly — implicitly, and asymmetrically, since the reverse
    /// comparison was exact.
    ///
    /// The subtraction is checked, so operands at opposite extremes of
    /// `Decimal`'s range report "not close" instead of panicking.
    ///
    /// For `Positive`-to-`Positive` approximate comparison use the `approx`
    /// traits ([`AbsDiffEq`] / [`RelativeEq`]), which this crate implements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{constants::EPSILON_CMP, pos_or_panic};
    /// use rust_decimal_macros::dec;
    ///
    /// let value = pos_or_panic!(1.0);
    /// assert!(value.approx_eq_dec(dec!(1.000000000000005), EPSILON_CMP));
    /// assert!(!value.approx_eq_dec(dec!(1.1), EPSILON_CMP));
    /// // exact equality disagrees, which is the point of having both
    /// assert!(value != dec!(1.000000000000005));
    /// ```
    #[inline]
    #[must_use]
    pub fn approx_eq_dec(&self, other: Decimal, epsilon: Decimal) -> bool {
        match dec_sub(self.0, other, "approx_eq_dec") {
            Ok(difference) => difference.abs() <= epsilon,
            Err(_) => false,
        }
    }

    /// Returns the smallest integer greater than or equal to the value.
    ///
    /// # Panics
    ///
    /// Panics when the result would break the positivity invariant. For a
    /// value that already satisfies the invariant the ceiling always does too,
    /// so in practice this method does not panic; the check is present so no
    /// `Positive`-returning path can bypass validation. Use
    /// [`Positive::checked_ceiling`] for the non-panicking form.
    #[inline]
    #[must_use]
    pub fn ceiling(&self) -> Positive {
        unwrap_or_panic(self.checked_ceiling(), "ceiling")
    }

    /// Returns the ceiling, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    ///
    /// assert_eq!(pos_or_panic!(1.1).checked_ceiling().unwrap(), pos_or_panic!(2.0));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_ceiling(&self) -> Result<Positive, PositiveError> {
        Positive::new_decimal(self.0.ceil())
    }

    /// Subtracts a decimal value, returning zero if the result would be negative.
    ///
    /// This method is not available when the `non-zero` feature is enabled
    /// because the result could be zero.
    ///
    /// Overflow cannot occur: the subtraction is performed with
    /// [`Decimal::checked_sub`], and an overflowing difference is treated the
    /// same as a negative one — the floor at zero is returned.
    #[cfg(not(feature = "non-zero"))]
    #[must_use]
    #[deprecated(
        since = "0.5.1",
        note = "saturating arithmetic hides underflow and overflow; use `checked_sub` or `checked_sub_dec` and handle the error, or explicitly floor at zero with `new_decimal(self.0.saturating_sub(*other))`. Removal is scheduled for the release after 0.6.0"
    )]
    pub fn sub_or_zero(&self, other: &Decimal) -> Positive {
        // Delegating to the checked path collapses the guard, the arithmetic
        // and the floor into one expression: a difference that is negative or
        // that overflows both come back as an error, and both floor at zero.
        self.checked_sub_dec(*other).unwrap_or(Positive::ZERO)
    }

    /// Subtracts a decimal value, returning `None` if the result would be
    /// negative.
    ///
    /// Also returns `None` when the subtraction would overflow `Decimal`, so
    /// this method cannot panic for any input.
    #[must_use]
    pub fn sub_or_none(&self, other: &Decimal) -> Option<Positive> {
        if &self.0 >= other {
            dec_sub(self.0, *other, "sub_or_none")
                .ok()
                .and_then(|result| Positive::new_decimal(result).ok())
        } else {
            None
        }
    }

    /// Checked addition of two `Positive` values.
    ///
    /// This is the non-panicking counterpart of the `+` operator.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the sum overflows
    /// `Decimal`, and [`PositiveError::OutOfBounds`] when the result would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError, pos_or_panic};
    /// use rust_decimal::Decimal;
    ///
    /// let a = pos_or_panic!(2.0);
    /// assert_eq!(a.checked_add(&pos_or_panic!(3.0)).unwrap(), pos_or_panic!(5.0));
    ///
    /// let max = Positive::new_decimal(Decimal::MAX).unwrap();
    /// assert!(matches!(
    ///     max.checked_add(&Positive::ONE).unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Positive::new_decimal(dec_add(self.0, rhs.0, "addition")?)
    }

    /// Checked multiplication of two `Positive` values.
    ///
    /// This is the non-panicking counterpart of the `*` operator.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the product overflows
    /// `Decimal`, and [`PositiveError::OutOfBounds`] when the result would
    /// break the positivity invariant — which, under the `non-zero` feature,
    /// includes a product that underflows to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError, pos_or_panic};
    /// use rust_decimal::Decimal;
    ///
    /// let a = pos_or_panic!(4.0);
    /// assert_eq!(a.checked_mul(&pos_or_panic!(2.5)).unwrap(), pos_or_panic!(10.0));
    ///
    /// let max = Positive::new_decimal(Decimal::MAX).unwrap();
    /// assert!(matches!(
    ///     max.checked_mul(&Positive::TWO).unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Positive::new_decimal(dec_mul(self.0, rhs.0, "multiplication")?)
    }

    /// Checked subtraction that returns a `Result` instead of panicking.
    ///
    /// This is the non-panicking counterpart of the `-` operator.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the difference
    /// overflows `Decimal`, and [`PositiveError::OutOfBounds`] when the result
    /// would be negative (or zero under the `non-zero` feature).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    ///
    /// let a = pos_or_panic!(5.0);
    /// assert_eq!(a.checked_sub(&pos_or_panic!(2.0)).unwrap(), pos_or_panic!(3.0));
    /// assert!(matches!(
    ///     a.checked_sub(&pos_or_panic!(9.0)).unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow/underflow error"]
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Positive::new_decimal(dec_sub(self.0, rhs.0, "subtraction")?)
    }

    /// Saturating subtraction that returns ZERO instead of negative.
    ///
    /// This method is not available when the `non-zero` feature is enabled
    /// because the result could be zero.
    ///
    /// # Deprecated
    ///
    /// `rules/global_rules.md` forbids saturating arithmetic: silently
    /// clamping an underflow to zero is indistinguishable from a genuine zero
    /// result, which in financial arithmetic is data corruption rather than a
    /// convenience. Use [`Positive::checked_sub`] and handle the error, or
    /// [`Positive::sub_or_zero`] if flooring at zero is genuinely what the
    /// caller wants and the intent should be visible at the call site.
    ///
    /// Scheduled for removal in the release following 0.6.0.
    #[cfg(not(feature = "non-zero"))]
    #[deprecated(
        since = "0.6.0",
        note = "saturating arithmetic hides underflow; use `checked_sub` and handle the error, or `sub_or_zero` to floor at zero explicitly. Removal is scheduled for the release after 0.6.0"
    )]
    #[must_use]
    pub fn saturating_sub(&self, rhs: &Self) -> Self {
        self.checked_sub(rhs).unwrap_or(Positive::ZERO)
    }

    /// Checked division that returns a `Result` instead of panicking.
    ///
    /// Uses [`DIV_ROUNDING_STRATEGY`] (banker's rounding) for any
    /// rounding required by the result. Use
    /// [`Positive::checked_div_with_strategy`] to select a different
    /// strategy.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on division by zero and on
    /// overflow — the latter is reachable, for example, when dividing
    /// `Decimal::MAX` by `1e-28`. Returns [`PositiveError::OutOfBounds`] when
    /// the quotient would break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError, pos_or_panic};
    /// use rust_decimal::Decimal;
    ///
    /// let a = pos_or_panic!(10.0);
    /// assert_eq!(a.checked_div(&pos_or_panic!(4.0)).unwrap(), pos_or_panic!(2.5));
    ///
    /// let max = Positive::new_decimal(Decimal::MAX).unwrap();
    /// let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    /// assert!(matches!(
    ///     max.checked_div(&tiny).unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the division-by-zero error"]
    pub fn checked_div(&self, rhs: &Self) -> Result<Self, PositiveError> {
        let quotient = dec_div(self.0, rhs.0, "division")?;
        Positive::new_decimal(round_div(quotient))
    }

    /// Checked division with an explicit rounding strategy.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on division by zero or
    /// overflow, and [`PositiveError::OutOfBounds`] when the quotient would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal::RoundingStrategy;
    ///
    /// let a = pos_or_panic!(10.0);
    /// let result = a
    ///     .checked_div_with_strategy(&pos_or_panic!(3.0), RoundingStrategy::ToZero)
    ///     .unwrap();
    /// assert!(result < pos_or_panic!(3.34));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_div_with_strategy(
        &self,
        rhs: &Self,
        strategy: RoundingStrategy,
    ) -> Result<Self, PositiveError> {
        let quotient = dec_div(self.0, rhs.0, "division")?;
        Positive::new_decimal(quotient.round_dp_with_strategy(DIV_SCALE, strategy))
    }

    /// Checked addition with a `Decimal`, returning a `Result` instead of
    /// panicking.
    ///
    /// This is the non-panicking counterpart of `Positive + Decimal`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on overflow and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant — for example when `rhs` is negative and larger in
    /// magnitude than `self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    /// use rust_decimal_macros::dec;
    ///
    /// let a = pos_or_panic!(5.0);
    /// assert_eq!(a.checked_add_dec(dec!(2.5)).unwrap(), pos_or_panic!(7.5));
    /// assert!(matches!(
    ///     a.checked_add_dec(dec!(-9)).unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_add_dec(self, rhs: Decimal) -> Result<Positive, PositiveError> {
        Positive::new_decimal(dec_add(self.0, rhs, "add_decimal")?)
    }

    /// Checked subtraction of a `Decimal`, returning a `Result` instead of
    /// panicking.
    ///
    /// This is the non-panicking counterpart of `Positive - Decimal`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on overflow and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// let a = pos_or_panic!(5.0);
    /// assert_eq!(a.checked_sub_dec(dec!(1.5)).unwrap(), pos_or_panic!(3.5));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_sub_dec(self, rhs: Decimal) -> Result<Positive, PositiveError> {
        Positive::new_decimal(dec_sub(self.0, rhs, "sub_decimal")?)
    }

    /// Checked multiplication by a `Decimal`, returning a `Result` instead of
    /// panicking.
    ///
    /// This is the non-panicking counterpart of `Positive * Decimal`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on overflow and
    /// [`PositiveError::OutOfBounds`] when the result would break the
    /// positivity invariant — for example when `rhs` is negative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// let a = pos_or_panic!(4.0);
    /// assert_eq!(a.checked_mul_dec(dec!(2.5)).unwrap(), pos_or_panic!(10.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_mul_dec(self, rhs: Decimal) -> Result<Positive, PositiveError> {
        Positive::new_decimal(dec_mul(self.0, rhs, "mul_decimal")?)
    }

    /// Checked division by a `Decimal`, returning a `Result` instead of
    /// panicking.
    ///
    /// This is the non-panicking counterpart of `Positive / Decimal`, and uses
    /// [`DIV_ROUNDING_STRATEGY`] when rounding is required.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on division by zero and on
    /// overflow, and [`PositiveError::OutOfBounds`] when the quotient would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    /// use rust_decimal::Decimal;
    /// use rust_decimal_macros::dec;
    ///
    /// let a = pos_or_panic!(10.0);
    /// assert_eq!(a.checked_div_dec(dec!(4)).unwrap(), pos_or_panic!(2.5));
    /// assert!(matches!(
    ///     a.checked_div_dec(Decimal::ZERO).unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_div_dec(self, rhs: Decimal) -> Result<Positive, PositiveError> {
        let quotient = dec_div(self.0, rhs, "div_decimal")?;
        Positive::new_decimal(round_div(quotient))
    }

    /// Checked remainder of division by another `Positive`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] on division by zero or
    /// overflow, and [`PositiveError::OutOfBounds`] when the remainder would
    /// break the positivity invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, pos_or_panic};
    ///
    /// let a = pos_or_panic!(10.0);
    /// assert_eq!(a.checked_rem(&pos_or_panic!(3.0)).unwrap(), pos_or_panic!(1.0));
    /// ```
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_rem(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Positive::new_decimal(dec_rem(self.0, rhs.0, "remainder")?)
    }

    /// Sums an iterator of `Positive` values without ever panicking.
    ///
    /// This is the aggregation counterpart of [`Positive::checked_add`], and
    /// the non-panicking alternative to the [`Sum`](std::iter::Sum)
    /// implementation. `Sum` cannot return a `Result`, so a fold that
    /// overflows has nowhere to report it; this function does.
    ///
    /// Accepts both owned and borrowed iterators — anything whose item
    /// `Borrow`s a `Positive` — so `values.iter()` and `values.into_iter()`
    /// both work.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] as soon as the running total
    /// overflows `Decimal`, without consuming the rest of the iterator.
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the total breaks the
    /// positivity invariant. Under the `non-zero` feature this includes the
    /// empty iterator, whose sum is zero: there is no valid `Positive`
    /// identity element, so an empty sum has no answer and is reported rather
    /// than invented.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError, pos_or_panic};
    /// use rust_decimal::Decimal;
    ///
    /// let values = [pos_or_panic!(1.5), pos_or_panic!(2.5), pos_or_panic!(6.0)];
    ///
    /// // borrowed
    /// assert_eq!(Positive::checked_sum(values.iter()).unwrap(), pos_or_panic!(10.0));
    /// // owned
    /// assert_eq!(Positive::checked_sum(values).unwrap(), pos_or_panic!(10.0));
    ///
    /// // overflow is reported, not panicked
    /// let max = Positive::new_decimal(Decimal::MAX).unwrap();
    /// assert!(matches!(
    ///     Positive::checked_sum([max, Positive::ONE]).unwrap_err(),
    ///     PositiveError::ArithmeticError { .. }
    /// ));
    /// ```
    #[must_use = "checked aggregation returns a Result; ignoring it silences the overflow error"]
    pub fn checked_sum<I, T>(iter: I) -> Result<Positive, PositiveError>
    where
        I: IntoIterator<Item = T>,
        T: Borrow<Positive>,
    {
        let mut total = Decimal::ZERO;
        for value in iter {
            total = dec_add(total, value.borrow().0, "sum")?;
        }
        Positive::new_decimal(total)
    }

    /// Checked addition with an `f64`, returning a `Result` instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns a `ConversionError` if `rhs` cannot be represented as a
    /// `Decimal` (e.g. NaN, `±inf`), an `ArithmeticError` on overflow, or
    /// an `OutOfBounds` if the result would violate the positivity
    /// invariant.
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_add_f64(self, rhs: f64) -> Result<Positive, PositiveError> {
        let rhs_dec = Decimal::from_f64(rhs).ok_or_else(|| {
            PositiveError::conversion_error("f64", "Decimal", "value not representable as Decimal")
        })?;
        Positive::new_decimal(dec_add(self.0, rhs_dec, "add_f64")?)
    }

    /// Checked subtraction with an `f64`, returning a `Result` instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns a `ConversionError` if `rhs` cannot be represented as a
    /// `Decimal`, an `ArithmeticError` on overflow, or an `OutOfBounds`
    /// if the result would violate the positivity invariant.
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_sub_f64(self, rhs: f64) -> Result<Positive, PositiveError> {
        let rhs_dec = Decimal::from_f64(rhs).ok_or_else(|| {
            PositiveError::conversion_error("f64", "Decimal", "value not representable as Decimal")
        })?;
        Positive::new_decimal(dec_sub(self.0, rhs_dec, "sub_f64")?)
    }

    /// Checked multiplication with an `f64`, returning a `Result` instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns a `ConversionError` if `rhs` cannot be represented as a
    /// `Decimal`, an `ArithmeticError` on overflow, or an `OutOfBounds`
    /// if the result would violate the positivity invariant (for example
    /// when `rhs` is negative).
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_mul_f64(self, rhs: f64) -> Result<Positive, PositiveError> {
        let rhs_dec = Decimal::from_f64(rhs).ok_or_else(|| {
            PositiveError::conversion_error("f64", "Decimal", "value not representable as Decimal")
        })?;
        Positive::new_decimal(dec_mul(self.0, rhs_dec, "mul_f64")?)
    }

    /// Checked division by an `f64`, returning a `Result` instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns a `ConversionError` if `rhs` cannot be represented as a
    /// `Decimal`, an `ArithmeticError` on overflow or division by zero,
    /// or an `OutOfBounds` if the result would violate the positivity
    /// invariant (for example when `rhs` is negative).
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_div_f64(self, rhs: f64) -> Result<Positive, PositiveError> {
        let rhs_dec = Decimal::from_f64(rhs).ok_or_else(|| {
            PositiveError::conversion_error("f64", "Decimal", "value not representable as Decimal")
        })?;
        Positive::new_decimal(round_div(dec_div(self.0, rhs_dec, "div_f64")?))
    }

    /// Checks whether the value is a multiple of another `f64` value.
    ///
    /// Prefer [`Positive::is_multiple_of_dec`] for full `Decimal` precision —
    /// this variant lifts the value to `f64` and compares against
    /// `f64::EPSILON`, which misclassifies values beyond the ~15 significant
    /// digits `f64` can carry.
    ///
    /// # Edge cases
    ///
    /// Until removal the contract is:
    ///
    /// - a zero divisor returns `false` — nothing is a multiple of zero;
    /// - a non-finite divisor (`NaN`, `±inf`) returns `false`;
    /// - a value that cannot be represented as a finite `f64` returns `false`
    ///   rather than panicking.
    ///
    /// # Deprecated
    ///
    /// Scheduled for removal in the release following 0.6.0. Use
    /// [`Positive::is_multiple_of_dec`] or [`Positive::is_multiple_of`], both
    /// of which are exact.
    #[deprecated(
        since = "0.5.0",
        note = "use `is_multiple_of_dec` for Decimal-native precision; removal is scheduled for the release after 0.6.0"
    )]
    #[must_use]
    pub fn is_multiple(&self, other: f64) -> bool {
        if !other.is_finite() || other == 0.0 {
            return false;
        }
        // `to_f64` panics for values outside f64's range; the checked form
        // reports them as "not a multiple" instead.
        let Some(value) = self.to_f64_checked() else {
            return false;
        };
        if !value.is_finite() {
            return false;
        }
        let remainder = value % other;
        remainder.abs() < f64::EPSILON || (other.abs() - remainder.abs()).abs() < f64::EPSILON
    }

    /// Checks whether the value is an exact multiple of a `Decimal`.
    ///
    /// The remainder must be exactly zero. Returns `false` when `other` is
    /// zero, since nothing is a multiple of zero. Uses a checked remainder, so
    /// no input can panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::pos_or_panic;
    /// use rust_decimal_macros::dec;
    ///
    /// assert!(pos_or_panic!(15.0).is_multiple_of_dec(dec!(5)));
    /// assert!(!pos_or_panic!(15.0).is_multiple_of_dec(dec!(4)));
    /// assert!(!pos_or_panic!(15.0).is_multiple_of_dec(dec!(0)));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_multiple_of_dec(&self, other: Decimal) -> bool {
        dec_rem(self.0, other, "is_multiple_of_dec")
            .map(|remainder| remainder.is_zero())
            .unwrap_or(false)
    }

    /// Checks whether the value is an exact multiple of another `Positive`.
    ///
    /// The remainder must be exactly zero, matching
    /// [`Positive::is_multiple_of_dec`] for the same divisor. Earlier versions
    /// compared the remainder against [`EPSILON`], so `1e-17` reported itself
    /// as a multiple of one — a false positive from a predicate that has an
    /// exact answer. Tolerance-based checking is available under the explicit
    /// name [`Positive::is_multiple_of_within`].
    ///
    /// Returns `false` when `other` is zero. Uses a checked remainder, so no
    /// input can panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, pos_or_panic};
    /// use rust_decimal::Decimal;
    ///
    /// assert!(pos_or_panic!(15.0).is_multiple_of(&pos_or_panic!(5.0)));
    ///
    /// let tiny = Positive::new_decimal(Decimal::new(1, 17)).unwrap();
    /// assert!(!tiny.is_multiple_of(&Positive::ONE));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_multiple_of(&self, other: &Positive) -> bool {
        self.is_multiple_of_dec(other.0)
    }

    /// Checks whether the value is a multiple of another `Positive` within an
    /// explicit tolerance.
    ///
    /// This is the tolerance-based counterpart of
    /// [`Positive::is_multiple_of`], which is exact. The tolerance is supplied
    /// by the caller rather than baked in, because the right tolerance depends
    /// on the magnitudes involved and on what the caller is modelling.
    ///
    /// A remainder is accepted when it is within `tolerance` of either zero or
    /// the divisor, so values just below an exact multiple count too.
    ///
    /// Returns `false` when `other` is zero. Cannot panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, pos_or_panic};
    /// use rust_decimal::Decimal;
    /// use rust_decimal_macros::dec;
    ///
    /// let tiny = Positive::new_decimal(Decimal::new(1, 17)).unwrap();
    /// assert!(!tiny.is_multiple_of(&Positive::ONE));
    /// assert!(tiny.is_multiple_of_within(&Positive::ONE, dec!(1e-16)));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_multiple_of_within(&self, other: &Positive, tolerance: Decimal) -> bool {
        let Ok(remainder) = dec_rem(self.0, other.0, "is_multiple_of_within") else {
            return false;
        };
        let distance_to_zero = remainder.abs();
        if distance_to_zero <= tolerance {
            return true;
        }
        match dec_sub(other.0, distance_to_zero, "is_multiple_of_within") {
            Ok(distance_to_divisor) => distance_to_divisor.abs() <= tolerance,
            Err(_) => false,
        }
    }

    /// Crate-private const constructor used exclusively by `crate::constants`
    /// to define `Positive` constants in `const` context.
    ///
    /// The invariant is enforced by the callers: every constant in
    /// `crate::constants` is a literal that is non-negative — strictly
    /// positive under the `non-zero` feature — and each is audited at the
    /// point of definition. Keeping this crate-private is what lets the
    /// constants exist at compile time without exposing an unchecked
    /// constructor to callers, which is why the public `new_unchecked` could
    /// be removed outright rather than replaced.
    #[inline]
    #[must_use]
    pub(crate) const fn from_decimal_const(value: Decimal) -> Self {
        Positive(value)
    }
}

/// Converts a `Positive` to an integer type, mapping the `None` that every
/// `Decimal::to_*` returns on overflow into a typed error.
///
/// The three `TryFrom<Positive>` impls differed only in the method they called
/// and the type name in the message, which is exactly the kind of duplication
/// that let `From<Positive> for usize` acquire a silent `unwrap_or(0)` while
/// its siblings did not.
#[inline]
fn try_to_integer<T>(
    value: Positive,
    convert: impl FnOnce(&Decimal) -> Option<T>,
    target: &'static str,
    reason: &'static str,
) -> Result<T, PositiveError> {
    convert(&value.0).ok_or_else(|| PositiveError::conversion_error("Positive", target, reason))
}

impl From<Positive> for Decimal {
    #[inline]
    fn from(value: Positive) -> Self {
        value.0
    }
}

impl PartialEq<&Positive> for Positive {
    #[inline]
    fn eq(&self, other: &&Positive) -> bool {
        self == *other
    }
}

impl TryFrom<Positive> for u64 {
    type Error = PositiveError;

    /// Converts a `Positive` to a `u64`, truncating any fraction toward zero.
    ///
    /// Replaces the previous `From<Positive> for u64`, which returned `0` when
    /// the value did not fit — conflating failure with a valid result.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// exceeds `u64::MAX`.
    #[inline]
    fn try_from(value: Positive) -> Result<Self, Self::Error> {
        try_to_integer(value, Decimal::to_u64, "u64", "value exceeds u64::MAX")
    }
}

impl TryFrom<Positive> for i64 {
    type Error = PositiveError;

    /// Converts a `Positive` to an `i64`, truncating any fraction toward zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// exceeds `i64::MAX`.
    #[inline]
    fn try_from(value: Positive) -> Result<Self, Self::Error> {
        try_to_integer(value, Decimal::to_i64, "i64", "value exceeds i64::MAX")
    }
}

impl From<&Positive> for f64 {
    /// Infallible, but lossy beyond `f64`'s ~15 significant digits. See
    /// [`Positive::to_f64`].
    #[inline]
    fn from(value: &Positive) -> Self {
        value.to_f64()
    }
}

impl From<Positive> for f64 {
    /// Infallible, but lossy beyond `f64`'s ~15 significant digits. See
    /// [`Positive::to_f64`].
    #[inline]
    fn from(value: Positive) -> Self {
        value.to_f64()
    }
}

impl TryFrom<Positive> for usize {
    type Error = PositiveError;

    /// Converts a `Positive` to a `usize`, truncating any fraction toward
    /// zero.
    ///
    /// Replaces the previous `From<Positive> for usize`, which went through
    /// `to_u64().unwrap_or(0)` and then cast to `usize` — so it both returned
    /// `0` for out-of-range values and silently wrapped on 32-bit targets.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// does not fit in a `usize` on the target platform.
    #[inline]
    fn try_from(value: Positive) -> Result<Self, Self::Error> {
        try_to_integer(
            value,
            Decimal::to_usize,
            "usize",
            "value exceeds usize::MAX",
        )
    }
}

impl PartialEq<&Positive> for f64 {
    #[inline]
    fn eq(&self, other: &&Positive) -> bool {
        cmp_decimal_f64(other.0, *self) == Some(Ordering::Equal)
    }
}

impl PartialOrd<&Positive> for f64 {
    #[inline]
    fn partial_cmp(&self, other: &&Positive) -> Option<Ordering> {
        cmp_decimal_f64(other.0, *self).map(Ordering::reverse)
    }
}

impl PartialEq<Positive> for f64 {
    #[inline]
    fn eq(&self, other: &Positive) -> bool {
        cmp_decimal_f64(other.0, *self) == Some(Ordering::Equal)
    }
}

impl PartialOrd<Positive> for f64 {
    #[inline]
    fn partial_cmp(&self, other: &Positive) -> Option<Ordering> {
        cmp_decimal_f64(other.0, *self).map(Ordering::reverse)
    }
}

impl Mul<Positive> for f64 {
    type Output = f64;
    #[inline]
    fn mul(self, rhs: Positive) -> Self::Output {
        self * rhs.to_f64()
    }
}

impl Div<Positive> for f64 {
    type Output = f64;
    #[inline]
    fn div(self, rhs: Positive) -> Self::Output {
        self / rhs.to_f64()
    }
}

impl Sub<Positive> for f64 {
    type Output = f64;
    #[inline]
    fn sub(self, rhs: Positive) -> Self::Output {
        self - rhs.to_f64()
    }
}

impl Add<Positive> for f64 {
    type Output = f64;
    #[inline]
    fn add(self, rhs: Positive) -> Self::Output {
        self + rhs.to_f64()
    }
}

impl FromStr for Positive {
    type Err = PositiveError;

    /// Parses a `Positive` from its decimal text representation.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidValue`] when `s` is not a well-formed
    /// decimal, carrying the offending input verbatim, and
    /// [`PositiveError::OutOfBounds`] when it parses but breaks the positivity
    /// invariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, PositiveError};
    /// use std::str::FromStr;
    ///
    /// assert!(Positive::from_str("1.5").is_ok());
    ///
    /// let err = Positive::from_str("not a number").unwrap_err();
    /// assert!(matches!(err, PositiveError::InvalidValue { .. }));
    /// assert!(err.to_string().contains("not a number"));
    ///
    /// assert!(matches!(
    ///     Positive::from_str("-1.5").unwrap_err(),
    ///     PositiveError::OutOfBounds { .. }
    /// ));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Decimal>() {
            Ok(value) => Positive::new_decimal(value),
            Err(e) => Err(PositiveError::invalid_value(
                s,
                &format!("failed to parse as decimal: {e}"),
            )),
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

    /// Converts a `usize` to a `Positive` exactly.
    ///
    /// The conversion goes straight to `Decimal`. Earlier versions went
    /// through `f64`, so on 64-bit targets every value above `2^53` was
    /// rounded — `9_007_199_254_740_993` became `9_007_199_254_740_992`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the value breaks the
    /// positivity invariant, which under the `non-zero` feature means zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::Positive;
    /// use rust_decimal::Decimal;
    ///
    /// let value = 9_007_199_254_740_993usize;
    /// let positive = Positive::try_from(value).unwrap();
    /// assert_eq!(positive.to_dec(), Decimal::from(value));
    /// ```
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Positive::new_decimal(Decimal::from(value))
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
    /// Copies an already-validated `Positive`, so the invariant holds by
    /// construction and no re-check is needed.
    #[inline]
    fn from(value: &Positive) -> Self {
        Positive(value.0)
    }
}

// `f64` operands are lifted to `Decimal` once, at the boundary, and then go
// through the same `Positive`/`Decimal` kernels as everything else. An `f64`
// that has no `Decimal` form — NaN, an infinity, or a magnitude outside the
// range — is reported the same way an invariant violation is, which is the
// contract these operators have always had.

impl Mul<f64> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics when `rhs` has no `Decimal` representation, on overflow, or
    /// when the product would break the positivity invariant. See
    /// [`Positive::checked_mul_f64`].
    #[inline]
    fn mul(self, rhs: f64) -> Positive {
        let rhs_dec = Decimal::from_f64(rhs).unwrap_or_else(|| invariant_panic("mul_f64"));
        unwrap_or_panic(self.checked_mul_dec(rhs_dec), "mul_f64")
    }
}

impl Div<f64> for Positive {
    type Output = Positive;
    /// Divides by an `f64` using [`DIV_ROUNDING_STRATEGY`] (banker's
    /// rounding) when rounding is required. For a different strategy
    /// use [`Positive::checked_div_with_strategy`] on the lifted
    /// `Decimal`.
    ///
    /// # Panics
    ///
    /// Panics when `rhs` has no `Decimal` representation, on division by
    /// zero, on overflow, or when the quotient would break the positivity
    /// invariant. See [`Positive::checked_div_f64`].
    #[inline]
    fn div(self, rhs: f64) -> Positive {
        let rhs_dec = Decimal::from_f64(rhs).unwrap_or_else(|| invariant_panic("div_f64"));
        guard_nonzero_divisor(rhs_dec, "div_f64");
        unwrap_or_panic(self.checked_div_dec(rhs_dec), "div_f64")
    }
}

impl Div<f64> for &Positive {
    type Output = Positive;
    /// Divides a `&Positive` by an `f64` using [`DIV_ROUNDING_STRATEGY`]
    /// (banker's rounding) when rounding is required.
    ///
    /// # Panics
    ///
    /// Same contract as `Positive / f64`.
    #[inline]
    fn div(self, rhs: f64) -> Positive {
        *self / rhs
    }
}

impl Sub<f64> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics when `rhs` has no `Decimal` representation, on overflow, or
    /// when the difference would break the positivity invariant. See
    /// [`Positive::checked_sub_f64`].
    #[inline]
    fn sub(self, rhs: f64) -> Self::Output {
        let rhs_dec = Decimal::from_f64(rhs).unwrap_or_else(|| invariant_panic("sub_f64"));
        unwrap_or_panic(self.checked_sub_dec(rhs_dec), "sub_f64")
    }
}

impl Add<f64> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics when `rhs` has no `Decimal` representation, on overflow, or
    /// when the sum would break the positivity invariant. See
    /// [`Positive::checked_add_f64`].
    #[inline]
    fn add(self, rhs: f64) -> Self::Output {
        let rhs_dec = Decimal::from_f64(rhs).unwrap_or_else(|| invariant_panic("add_f64"));
        unwrap_or_panic(self.checked_add_dec(rhs_dec), "add_f64")
    }
}

impl PartialOrd<f64> for Positive {
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        cmp_decimal_f64(self.0, *other)
    }
}

impl PartialEq<f64> for &Positive {
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        cmp_decimal_f64(self.0, *other) == Some(Ordering::Equal)
    }
}

impl PartialOrd<f64> for &Positive {
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        cmp_decimal_f64(self.0, *other)
    }
}

impl PartialEq<f64> for Positive {
    /// Exact equality against an `f64`, symmetric with `f64 == Positive`.
    ///
    /// The previous implementation went through [`Positive::to_f64`], which
    /// panics for values outside `f64`'s range, and collapsed distinct decimal
    /// integers above `2^53` onto the same float.
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        cmp_decimal_f64(self.0, *other) == Some(Ordering::Equal)
    }
}

impl Display for Positive {
    /// Renders the underlying `Decimal` exactly.
    ///
    /// Earlier versions special-cased `Positive::INFINITY` and printed
    /// `f64::MAX` — a value roughly 10^279 times larger than the one the type
    /// actually held, and one that `Positive::new` rejects.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            return write!(f, "{:.1$}", self.0, precision);
        }
        // `Decimal::normalize` strips trailing zeros past the decimal
        // point (e.g. `1.500` -> `1.5`, `5.00` -> `5`), which matches
        // the previous `to_string() + trim_end_matches('0')` approach
        // without allocating an intermediate `String`.
        write!(f, "{}", self.0.normalize())
    }
}

impl fmt::Debug for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Same normalisation as `Display` so integer-valued decimals
        // render without trailing `.0` and fractional ones without
        // trailing zeros.
        write!(f, "{}", self.0.normalize())
    }
}

impl PartialEq<Decimal> for Positive {
    /// Exact equality, symmetric with `Decimal == Positive`.
    ///
    /// Earlier versions compared `|self - other| <= EPSILON_CMP` here while the
    /// reverse impl compared exactly, so `positive == decimal` and `decimal ==
    /// positive` could disagree — `PartialEq` requires them to agree. The
    /// subtraction also overflowed and panicked at the extremes of `Decimal`'s
    /// range.
    ///
    /// Approximate comparison now lives exclusively in the `approx` impls and
    /// in [`Positive::approx_eq_dec`].
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        self.0 == *other
    }
}

// Wire format (#75).
//
// `Positive` serialises as the **exact decimal string** produced by
// `Decimal`'s own `Serialize`, e.g. `"12.345"`, `"42"`,
// `"0.1234567890123456789012345678"`.
//
// The previous format converted every fractional value through `f64` and every
// scale-zero value through `i64`. Both are lossy or outright failing for
// values this crate is built to carry:
//
//   - `0.1234567890123456789012345678` serialised as `0.12345678901234569` and
//     came back as `0.1234567890123457` — 12 digits lost by a crate whose
//     whole point is 28-digit precision;
//   - `9223372036854775808` is a perfectly valid `Positive` but serialisation
//     failed outright with "Failed to convert to i64";
//   - `Decimal::MAX` could not be serialised at all.
//
// A JSON *number* cannot carry this precision: almost every consumer parses it
// into an f64. A string can, in every format, which is why `Decimal` itself
// uses one. Deserialisation still accepts JSON numbers so documents written by
// 0.5.x keep loading, but those are lossy by construction — the precision was
// already gone before the bytes reached us.
//
// Using `deserialize_str` rather than `deserialize_any` also means the format
// works with non-self-describing serializers, which `deserialize_any` cannot
// support.
impl Serialize for Positive {
    /// Serialises the exact decimal representation as a string.
    ///
    /// # Errors
    ///
    /// Propagates any error from the serializer. The conversion itself cannot
    /// fail: every `Positive` has an exact decimal representation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Disambiguated: `Decimal` also has an inherent `serialize()` that
        // returns its 16-byte representation.
        Serialize::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Positive {
    /// Deserialises from the exact decimal string, or from a JSON number for
    /// compatibility with documents written by 0.5.x.
    ///
    /// # Errors
    ///
    /// Fails when the input is not a valid decimal, or when the value breaks
    /// the positivity invariant — which under the `non-zero` feature includes
    /// zero.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositiveVisitor;

        impl Visitor<'_> for PositiveVisitor {
            type Value = Positive;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a positive decimal, as a string or a number")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decimal = Decimal::from_str_exact(value).map_err(|error| {
                    serde::de::Error::custom(format!("invalid decimal string '{value}': {error}"))
                })?;
                Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Positive::new_decimal(Decimal::from(value)).map_err(serde::de::Error::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Positive::new_decimal(Decimal::from(value)).map_err(serde::de::Error::custom)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // Legacy numeric input only. The precision was already lost
                // before these bytes reached us; nothing here can recover it.
                let decimal = Decimal::from_f64(value).ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "number {value} is not representable as a decimal"
                    ))
                })?;
                Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
            }
        }

        // Human-readable formats (JSON, YAML, TOML) are self-describing, so
        // `deserialize_any` can dispatch on the token that is actually there.
        // That is what keeps documents written by 0.5.x — which stored plain
        // JSON numbers — loading.
        //
        // Non-self-describing formats (bincode, postcard) cannot support
        // `deserialize_any` at all; they need to be told which type to read.
        // Since serialisation always emits a string, `deserialize_str` is the
        // matching request. Choosing between the two per format is what makes
        // the contract work in both worlds, rather than sacrificing one.
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(PositiveVisitor)
        } else {
            deserializer.deserialize_str(PositiveVisitor)
        }
    }
}

// ===========================================================================
// Operator adapters
// ===========================================================================
//
// Every operator below is a thin adapter. The arithmetic, the overflow
// mapping and the invariant check all live in the `checked_*` methods and the
// `dec_*` kernels; an operator's only job is to pick the kernel and convert
// its typed error into the documented panic.
//
// Written out longhand, each of these was six to ten lines of `match ...
// checked_op` with its own literal operation name, repeated for the owned and
// reference forms of three operand types. That duplication is what let
// `checked_div` drift onto raw division while `checked_div_with_strategy`
// used the checked one, and what let `Positive * Positive` skip the invariant
// check that `Positive * Decimal` performed.

/// Rejects a zero divisor before an operator divides.
///
/// Kept separate from the kernels because the operators report a zero divisor
/// as an invariant violation rather than an arithmetic error, and that
/// distinction is part of their documented panic messages.
#[inline]
fn guard_nonzero_divisor(divisor: Decimal, op: &'static str) {
    if divisor.is_zero() {
        invariant_panic(op);
    }
}

/// Unwraps a `Decimal`-valued kernel result for an operator whose output is a
/// `Decimal` rather than a `Positive`.
#[inline]
fn dec_or_panic(result: Result<Decimal, PositiveError>, op: &'static str) -> Decimal {
    match result {
        Ok(value) => value,
        Err(_) => overflow_panic(op),
    }
}

impl Add for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow. See [`Positive::checked_add`] for the
    /// non-panicking form.
    #[inline]
    fn add(self, other: Positive) -> Positive {
        unwrap_or_panic(self.checked_add(&other), "add")
    }
}

impl Sub for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow, or when the difference would be negative — zero
    /// under the `non-zero` feature. See [`Positive::checked_sub`].
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unwrap_or_panic(self.checked_sub(&rhs), "sub")
    }
}

impl Mul for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow, and — under the `non-zero` feature — when the
    /// product underflows to zero, as `1e-28 * 1e-28` does. See
    /// [`Positive::checked_mul`] for the non-panicking form.
    #[inline]
    fn mul(self, other: Positive) -> Positive {
        unwrap_or_panic(self.checked_mul(&other), "mul")
    }
}

impl Div for Positive {
    type Output = Positive;
    /// Divides two `Positive` values using [`DIV_ROUNDING_STRATEGY`]
    /// (banker's rounding) when rounding is required. For a different
    /// strategy use [`Positive::checked_div_with_strategy`].
    ///
    /// # Panics
    ///
    /// Panics on division by zero, on overflow, and when the quotient would
    /// break the positivity invariant.
    #[inline]
    fn div(self, other: Positive) -> Self::Output {
        guard_nonzero_divisor(other.0, "div");
        unwrap_or_panic(self.checked_div(&other), "div")
    }
}

impl Div for &Positive {
    type Output = Positive;
    /// Divides two `&Positive` values using [`DIV_ROUNDING_STRATEGY`]
    /// (banker's rounding) when rounding is required.
    ///
    /// # Panics
    ///
    /// Same contract as `Positive / Positive`; both delegate to
    /// [`Positive::checked_div`].
    #[inline]
    fn div(self, other: &Positive) -> Self::Output {
        guard_nonzero_divisor(other.0, "div");
        unwrap_or_panic(self.checked_div(other), "div")
    }
}

impl Add<Decimal> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow, or when the sum would break the positivity
    /// invariant. See [`Positive::checked_add_dec`].
    #[inline]
    fn add(self, rhs: Decimal) -> Positive {
        unwrap_or_panic(self.checked_add_dec(rhs), "add_decimal")
    }
}

impl Add<&Decimal> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Same contract as `Positive + Decimal`.
    #[inline]
    fn add(self, rhs: &Decimal) -> Self::Output {
        unwrap_or_panic(self.checked_add_dec(*rhs), "add_decimal")
    }
}

impl Sub<Decimal> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow, or when the difference would break the positivity
    /// invariant. See [`Positive::checked_sub_dec`].
    #[inline]
    fn sub(self, rhs: Decimal) -> Positive {
        unwrap_or_panic(self.checked_sub_dec(rhs), "sub_decimal")
    }
}

impl Sub<&Decimal> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Same contract as `Positive - Decimal`.
    #[inline]
    fn sub(self, rhs: &Decimal) -> Self::Output {
        unwrap_or_panic(self.checked_sub_dec(*rhs), "sub_decimal")
    }
}

impl Mul<Decimal> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Panics on overflow, or when the product would break the positivity
    /// invariant — for example when `rhs` is negative. See
    /// [`Positive::checked_mul_dec`].
    #[inline]
    fn mul(self, rhs: Decimal) -> Positive {
        unwrap_or_panic(self.checked_mul_dec(rhs), "mul_decimal")
    }
}

impl Div<Decimal> for Positive {
    type Output = Positive;
    /// Divides by a `Decimal` using [`DIV_ROUNDING_STRATEGY`] (banker's
    /// rounding) when rounding is required.
    ///
    /// # Panics
    ///
    /// Panics on division by zero, on overflow, and when the quotient would
    /// break the positivity invariant. See [`Positive::checked_div_dec`].
    #[inline]
    fn div(self, rhs: Decimal) -> Positive {
        guard_nonzero_divisor(rhs, "div_decimal");
        unwrap_or_panic(self.checked_div_dec(rhs), "div_decimal")
    }
}

impl Div<&Decimal> for Positive {
    type Output = Positive;
    /// Divides by a `&Decimal` using [`DIV_ROUNDING_STRATEGY`] (banker's
    /// rounding) when rounding is required.
    ///
    /// # Panics
    ///
    /// Same contract as `Positive / Decimal`.
    #[inline]
    fn div(self, rhs: &Decimal) -> Self::Output {
        guard_nonzero_divisor(*rhs, "div_decimal");
        unwrap_or_panic(self.checked_div_dec(*rhs), "div_decimal")
    }
}

impl AddAssign for Positive {
    /// # Panics
    ///
    /// Panics on overflow, or when the result would break the positivity
    /// invariant.
    #[inline]
    fn add_assign(&mut self, other: Positive) {
        *self = unwrap_or_panic(self.checked_add(&other), "add_assign");
    }
}

impl AddAssign<Decimal> for Positive {
    /// # Panics
    ///
    /// Panics on overflow, or when the result would break the positivity
    /// invariant.
    #[inline]
    fn add_assign(&mut self, rhs: Decimal) {
        *self = unwrap_or_panic(self.checked_add_dec(rhs), "add_assign_decimal");
    }
}

impl MulAssign<Decimal> for Positive {
    /// # Panics
    ///
    /// Panics on overflow, or when the result would break the positivity
    /// invariant.
    #[inline]
    fn mul_assign(&mut self, rhs: Decimal) {
        *self = unwrap_or_panic(self.checked_mul_dec(rhs), "mul_assign_decimal");
    }
}

impl PartialOrd<Decimal> for Positive {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<Positive> for Decimal {
    /// Mirror of `Positive: PartialOrd<Decimal>`, so ordering is available in
    /// both directions and agrees.
    #[inline]
    fn partial_cmp(&self, other: &Positive) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

// --- `Decimal` on the left-hand side: the result is a `Decimal`, so these go
// --- through the `dec_*` kernels directly rather than through `Positive`'s
// --- checked methods, which would validate an invariant the output does not
// --- have to satisfy.

impl Mul<Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn mul(self, rhs: Positive) -> Decimal {
        dec_or_panic(
            dec_mul(self, rhs.0, "mul_decimal_by_positive"),
            "mul_decimal_by_positive",
        )
    }
}

impl Div<Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Panics on division by zero and on overflow.
    #[inline]
    fn div(self, rhs: Positive) -> Decimal {
        guard_nonzero_divisor(rhs.0, "div_decimal_by_positive");
        dec_or_panic(
            dec_div(self, rhs.0, "div_decimal_by_positive"),
            "div_decimal_by_positive",
        )
    }
}

impl Sub<Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn sub(self, rhs: Positive) -> Decimal {
        dec_or_panic(
            dec_sub(self, rhs.0, "sub_decimal_by_positive"),
            "sub_decimal_by_positive",
        )
    }
}

impl Sub<&Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Same contract as `Decimal - Positive`.
    #[inline]
    fn sub(self, rhs: &Positive) -> Decimal {
        self - *rhs
    }
}

impl Add<Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn add(self, rhs: Positive) -> Decimal {
        dec_or_panic(
            dec_add(self, rhs.0, "add_decimal_by_positive"),
            "add_decimal_by_positive",
        )
    }
}

impl Add<&Positive> for Decimal {
    type Output = Decimal;
    /// # Panics
    ///
    /// Same contract as `Decimal + Positive`.
    #[inline]
    fn add(self, rhs: &Positive) -> Decimal {
        self + *rhs
    }
}

impl std::ops::AddAssign<Positive> for Decimal {
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn add_assign(&mut self, rhs: Positive) {
        *self = dec_or_panic(
            dec_add(*self, rhs.0, "add_assign_decimal_by_positive"),
            "add_assign_decimal_by_positive",
        );
    }
}

impl std::ops::AddAssign<&Positive> for Decimal {
    /// # Panics
    ///
    /// Same contract as `Decimal += Positive`.
    #[inline]
    fn add_assign(&mut self, rhs: &Positive) {
        *self += *rhs;
    }
}

impl std::ops::MulAssign<Positive> for Decimal {
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn mul_assign(&mut self, rhs: Positive) {
        *self = dec_or_panic(
            dec_mul(*self, rhs.0, "mul_assign_decimal_by_positive"),
            "mul_assign_decimal_by_positive",
        );
    }
}

impl std::ops::MulAssign<&Positive> for Decimal {
    /// # Panics
    ///
    /// Same contract as `Decimal *= Positive`.
    #[inline]
    fn mul_assign(&mut self, rhs: &Positive) {
        *self *= *rhs;
    }
}

impl PartialEq<Positive> for Decimal {
    #[inline]
    fn eq(&self, other: &Positive) -> bool {
        *self == other.0
    }
}

impl From<&Positive> for Decimal {
    #[inline]
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
        // A difference too large to represent cannot be within any epsilon.
        match dec_sub(self.0, other.0, "abs_diff_eq") {
            Ok(difference) => difference.abs() <= epsilon,
            Err(_) => false,
        }
    }
}

impl RelativeEq for Positive {
    fn default_max_relative() -> Self::Epsilon {
        EPSILON_CMP
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        let Ok(difference) = dec_sub(self.0, other.0, "relative_eq") else {
            // A difference too large to represent cannot be within any
            // absolute epsilon, and the relative test below would need a
            // tolerance larger than `Decimal::MAX` to accept it.
            return false;
        };
        let abs_diff = difference.abs();
        if abs_diff <= epsilon {
            return true;
        }
        if max_relative.is_sign_negative() {
            // Negative tolerance makes no sense; reject the comparison.
            return false;
        }
        let largest = self.0.abs().max(other.0.abs());
        match dec_mul(max_relative, largest, "relative_eq") {
            Ok(tolerance) => abs_diff <= tolerance,
            // The tolerance overflowed `Decimal`, so it exceeds every
            // representable difference — including this one.
            Err(_) => true,
        }
    }
}

// `Sum` cannot return a `Result`, so it is the one aggregation path that has
// to fail loudly. Both impls delegate to `Positive::checked_sum` and convert
// its error into the documented panic. The previous implementation folded with
// raw `Decimal` addition — which panicked inside rust_decimal on overflow
// anyway — and then applied `unwrap_or(Positive::ZERO)`, which could never
// observe that overflow and would have replaced a financial total with zero if
// it ever had.
//
// Without the `non-zero` feature the sum of non-negative values is itself
// non-negative, so overflow is the only reachable failure.

#[cfg(not(feature = "non-zero"))]
impl Sum for Positive {
    /// Sums an iterator of `Positive` values.
    ///
    /// # Panics
    ///
    /// Panics when the total overflows `Decimal`. Use
    /// [`Positive::checked_sum`] for the non-panicking form; it reports the
    /// overflow as a [`PositiveError::ArithmeticError`] instead.
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        match Positive::checked_sum(iter) {
            Ok(total) => total,
            Err(_) => overflow_panic("sum"),
        }
    }
}

#[cfg(not(feature = "non-zero"))]
impl<'a> Sum<&'a Positive> for Positive {
    /// Sums an iterator of `&Positive` values.
    ///
    /// # Panics
    ///
    /// Panics when the total overflows `Decimal`. Use
    /// [`Positive::checked_sum`] for the non-panicking form.
    fn sum<I: Iterator<Item = &'a Positive>>(iter: I) -> Self {
        match Positive::checked_sum(iter) {
            Ok(total) => total,
            Err(_) => overflow_panic("sum"),
        }
    }
}
