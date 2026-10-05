/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 05/10/26
******************************************************************************/

//! The [`StrictlyPositive`] type: a decimal that is always greater than zero.
//!
//! `StrictlyPositive` sits next to [`Positive`] rather than replacing it, so a
//! crate can hold both `> 0` and `>= 0` values at once.
//!
//! # Design
//!
//! The type is a `#[repr(transparent)]` newtype over `Positive`, so its layout
//! is exactly that of a `Decimal`. Every operation delegates to the matching
//! `Positive` method and then re-checks the strict invariant in one place,
//! [`StrictlyPositive::lift`]. The arithmetic, the rounding and the overflow
//! handling therefore cannot drift between the two types.
//!
//! # Errors
//!
//! A value that is zero or negative is reported as
//! [`PositiveError::OutOfBounds`] whose `min` is [`StrictlyPositive::MIN`]
//! (`1e-28`, the smallest strictly positive `Decimal`) and whose `max` is
//! `Decimal::MAX`, not the `0` minimum that [`Positive`] reports.

use crate::error::PositiveError;
use crate::positive::{
    Positive, dec_add, overflow_panic, precision_panic, unwrap_or_panic, validate_precision,
};
use approx::{AbsDiffEq, RelativeEq};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};
use std::str::FromStr;

/// A decimal value that is guaranteed to be strictly greater than zero.
///
/// Where [`Positive`] means `>= 0`, `StrictlyPositive` means `> 0`. Use it for
/// quantities where zero is as invalid as a negative number, such as prices,
/// divisors or volatilities, and keep `Positive` for quantities that may be
/// zero.
///
/// # Examples
///
/// ```rust
/// use positive::{Positive, PositiveError, StrictlyPositive};
/// use rust_decimal_macros::dec;
///
/// struct Instrument {
///     price: StrictlyPositive, // > 0
///     daily_volume: Positive,  // >= 0
/// }
///
/// # fn main() -> Result<(), PositiveError> {
/// let instrument = Instrument {
///     price: StrictlyPositive::new_decimal(dec!(101.25))?,
///     daily_volume: Positive::new_decimal(dec!(1500))?,
/// };
/// assert!(instrument.price > dec!(0));
/// assert!(StrictlyPositive::new_decimal(dec!(0)).is_err());
/// # let _ = instrument.daily_volume;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StrictlyPositive(Positive);

/// Panics with a uniform message when an operation on `StrictlyPositive`
/// would produce a value that is zero or negative.
#[cold]
#[inline(never)]
fn strict_invariant_panic(op: &'static str) -> ! {
    panic!("StrictlyPositive invariant broken in {op}: result would be zero or negative")
}

/// Panics with a uniform message when a clamp range is inverted.
#[cold]
#[inline(never)]
fn strict_inverted_range_panic() -> ! {
    panic!("StrictlyPositive clamp range is inverted: min is greater than max")
}

/// Builds the error reported for a value that breaks the strict invariant.
#[cold]
#[inline(never)]
fn strict_out_of_bounds(value: Decimal) -> PositiveError {
    PositiveError::out_of_bounds(value, StrictlyPositive::MIN.to_dec(), Decimal::MAX)
}

/// Converts a checked result into the panic documented on the panicking
/// wrapper. It is the single point at which a typed error on a
/// `StrictlyPositive` operation becomes a panic.
#[inline]
fn strict_unwrap_or_panic(
    result: Result<StrictlyPositive, PositiveError>,
    op: &'static str,
) -> StrictlyPositive {
    match result {
        Ok(value) => value,
        Err(PositiveError::OutOfBounds { .. }) => strict_invariant_panic(op),
        Err(PositiveError::ArithmeticError { .. }) => overflow_panic(op),
        Err(PositiveError::InvalidPrecision { precision, .. }) => precision_panic(precision),
        Err(
            error @ (PositiveError::InvalidValue { .. } | PositiveError::ConversionError { .. }),
        ) => panic!("StrictlyPositive {op} failed: {error}"),
    }
}

impl StrictlyPositive {
    /// The smallest value a `StrictlyPositive` can hold: `1e-28`, the smallest
    /// strictly positive `Decimal`.
    ///
    /// This is also the `min` reported by [`PositiveError::OutOfBounds`] when
    /// a value is rejected for being zero or negative.
    pub const MIN: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(1e-28));
    /// The largest value a `StrictlyPositive` can hold: `Decimal::MAX`.
    pub const MAX: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::MAX);
    /// The value one.
    pub const ONE: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::ONE);
    /// The value two.
    pub const TWO: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::TWO);
    /// The value three.
    pub const THREE: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(3));
    /// The value four.
    pub const FOUR: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(4));
    /// The value five.
    pub const FIVE: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(5));
    /// The value six.
    pub const SIX: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(6));
    /// The value seven.
    pub const SEVEN: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(7));
    /// The value eight.
    pub const EIGHT: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(8));
    /// The value nine.
    pub const NINE: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(9));
    /// The value ten.
    pub const TEN: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::TEN);
    /// The value one hundred.
    pub const HUNDRED: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(100));
    /// The value one thousand.
    pub const THOUSAND: StrictlyPositive = StrictlyPositive::from_decimal_const(dec!(1000));
    /// The mathematical constant π, at `Decimal` precision.
    pub const PI: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::PI);
    /// Euler's number e, at `Decimal` precision.
    pub const E: StrictlyPositive = StrictlyPositive::from_decimal_const(Decimal::E);

    /// Stores a literal without validation, so constants can be built at
    /// compile time.
    ///
    /// Crate-private on purpose: every caller passes a literal that is
    /// strictly positive, which satisfies the `Positive` invariant as well as
    /// this type's own.
    #[inline]
    #[must_use]
    const fn from_decimal_const(value: Decimal) -> Self {
        StrictlyPositive(Positive::from_decimal_const(value))
    }

    /// Wraps an already validated `Positive`, rejecting zero.
    ///
    /// Every path that yields a `StrictlyPositive` ends here, so the strict
    /// invariant is checked in exactly one place.
    #[inline]
    fn from_positive_checked(value: Positive) -> Result<Self, PositiveError> {
        if value.is_zero() {
            Err(strict_out_of_bounds(value.to_dec()))
        } else {
            Ok(StrictlyPositive(value))
        }
    }

    /// Lifts the result of a `Positive` operation into a `StrictlyPositive`.
    ///
    /// An `OutOfBounds` error from `Positive` (whose minimum is `0`) is
    /// re-reported with this type's bounds (minimum `1e-28`), so the caller
    /// sees the bounds of the type it asked for. Every other error passes
    /// through unchanged. `Deserialize` cannot use this (serde errors are
    /// opaque), see its docs.
    #[inline]
    fn lift(result: Result<Positive, PositiveError>) -> Result<Self, PositiveError> {
        match result {
            Ok(value) => Self::from_positive_checked(value),
            Err(PositiveError::OutOfBounds { value, .. }) => Err(strict_out_of_bounds(value)),
            Err(error) => Err(error),
        }
    }

    /// Creates a `StrictlyPositive` from a 64-bit floating-point number.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidValue`] when `value` is `NaN` or
    /// infinite, [`PositiveError::ConversionError`] when it is outside the
    /// range `Decimal` can represent, and [`PositiveError::OutOfBounds`] when
    /// it is zero or negative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert!(StrictlyPositive::new(1.5).is_ok());
    /// assert!(matches!(
    ///     StrictlyPositive::new(0.0),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[must_use = "constructor returns a Result; ignoring it discards a validated invariant"]
    pub fn new(value: f64) -> Result<Self, PositiveError> {
        Self::lift(Positive::new(value))
    }

    /// Creates a `StrictlyPositive` from a `Decimal`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero or
    /// negative. The reported value and bounds are exact `Decimal`s.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    /// use rust_decimal_macros::dec;
    ///
    /// assert!(StrictlyPositive::new_decimal(dec!(0.01)).is_ok());
    /// assert!(matches!(
    ///     StrictlyPositive::new_decimal(dec!(0)),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[must_use = "constructor returns a Result; ignoring it discards a validated invariant"]
    pub fn new_decimal(value: Decimal) -> Result<Self, PositiveError> {
        if value > Decimal::ZERO {
            Ok(StrictlyPositive::from_decimal_const(value))
        } else {
            Err(strict_out_of_bounds(value))
        }
    }

    /// Returns the inner value as a `Decimal`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::TWO.value(), dec!(2));
    /// ```
    #[inline]
    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0.to_dec()
    }

    /// Returns the inner value as a `Decimal` (alias for [`Self::value`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::TWO.to_dec(), dec!(2));
    /// ```
    #[inline]
    #[must_use]
    pub fn to_dec(&self) -> Decimal {
        self.0.to_dec()
    }

    /// Returns the value as a [`Positive`]. This never fails: every strictly
    /// positive value is also positive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.to_positive(), Positive::TWO);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_positive(&self) -> Positive {
        self.0
    }

    /// Converts the value to an `f64`.
    ///
    /// Infallible but lossy beyond `f64`'s ~15 significant digits; see
    /// [`Positive::to_f64`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TWO.to_f64(), 2.0);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64()
    }

    /// Converts the value to an `i64`, truncating toward zero. Returns `None`
    /// when the truncated value exceeds `i64::MAX`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TEN.to_i64_checked(), Some(10));
    /// assert_eq!(StrictlyPositive::MAX.to_i64_checked(), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_i64_checked(&self) -> Option<i64> {
        self.0.to_i64_checked()
    }

    /// Converts the value to a `u64`, truncating toward zero. Returns `None`
    /// when the truncated value exceeds `u64::MAX`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TEN.to_u64_checked(), Some(10));
    /// ```
    #[inline]
    #[must_use]
    pub fn to_u64_checked(&self) -> Option<u64> {
        self.0.to_u64_checked()
    }

    /// Converts the value to a `usize`, truncating toward zero. Returns
    /// `None` when the truncated value exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TEN.to_usize_checked(), Some(10));
    /// ```
    #[inline]
    #[must_use]
    pub fn to_usize_checked(&self) -> Option<usize> {
        self.0.to_usize_checked()
    }

    // -----------------------------------------------------------------------
    // Checked arithmetic
    // -----------------------------------------------------------------------

    /// Adds two `StrictlyPositive` values. Non-panicking counterpart of `+`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the sum overflows
    /// `Decimal`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_add(&StrictlyPositive::ONE), Ok(StrictlyPositive::THREE));
    /// assert!(matches!(
    ///     StrictlyPositive::MAX.checked_add(&StrictlyPositive::ONE),
    ///     Err(PositiveError::ArithmeticError { .. })
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_add(&rhs.0))
    }

    /// Adds a [`Positive`] to this value. The sum of a strictly positive and
    /// a non-negative value is strictly positive, so the result keeps this
    /// type. Non-panicking counterpart of `StrictlyPositive + Positive`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the sum overflows
    /// `Decimal`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// let sum = StrictlyPositive::ONE.checked_add_positive(&Positive::TWO);
    /// assert_eq!(sum, Ok(StrictlyPositive::THREE));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_add_positive(&self, rhs: &Positive) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_add(rhs))
    }

    /// Subtracts `rhs`, requiring the difference to stay strictly positive.
    ///
    /// The `-` operator returns a [`Positive`] instead, because the
    /// difference of two strictly positive values may be zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the difference is zero or
    /// negative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::THREE.checked_sub(&StrictlyPositive::ONE), Ok(StrictlyPositive::TWO));
    /// assert!(matches!(
    ///     StrictlyPositive::ONE.checked_sub(&StrictlyPositive::ONE),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_sub(&rhs.0))
    }

    /// Multiplies two `StrictlyPositive` values. Non-panicking counterpart of
    /// `*`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the product overflows
    /// `Decimal`, and [`PositiveError::OutOfBounds`] when it underflows to
    /// zero, as `1e-20 * 1e-20` does: `Decimal` keeps at most 28 decimal
    /// places.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_mul(&StrictlyPositive::THREE), Ok(StrictlyPositive::SIX));
    ///
    /// let tiny = StrictlyPositive::new_decimal(dec!(1e-20))?;
    /// assert!(matches!(tiny.checked_mul(&tiny), Err(PositiveError::OutOfBounds { .. })));
    /// # Ok::<(), PositiveError>(())
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_mul(&rhs.0))
    }

    /// Multiplies by a [`Positive`]. The result is a `Positive`, because the
    /// factor may be zero. Non-panicking counterpart of
    /// `StrictlyPositive * Positive`.
    ///
    /// # Errors
    ///
    /// Same as [`Positive::checked_mul`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_mul_positive(&Positive::THREE), Ok(Positive::SIX));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the overflow error"]
    pub fn checked_mul_positive(&self, rhs: &Positive) -> Result<Positive, PositiveError> {
        self.0.checked_mul(rhs)
    }

    /// Divides two `StrictlyPositive` values. Non-panicking counterpart of
    /// `/`.
    ///
    /// Rounds with [`crate::DIV_ROUNDING_STRATEGY`] (banker's rounding) at 28
    /// decimal places, exactly as [`Positive::checked_div`] does. The divisor
    /// cannot be zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the quotient overflows
    /// `Decimal`, and [`PositiveError::OutOfBounds`] when it underflows to
    /// zero, as `1e-28 / 10` does.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::SIX.checked_div(&StrictlyPositive::TWO), Ok(StrictlyPositive::THREE));
    /// assert!(matches!(
    ///     StrictlyPositive::MIN.checked_div(&StrictlyPositive::TEN),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the underflow error"]
    pub fn checked_div(&self, rhs: &Self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_div(&rhs.0))
    }

    /// Divides by a [`Positive`], returning a `Positive`. Non-panicking
    /// counterpart of `StrictlyPositive / Positive`.
    ///
    /// Rounds with [`crate::DIV_ROUNDING_STRATEGY`], like
    /// [`Positive::checked_div`].
    ///
    /// # Errors
    ///
    /// Same as [`Positive::checked_div`]; in particular, a zero divisor
    /// returns [`PositiveError::ArithmeticError`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::SIX.checked_div_positive(&Positive::TWO), Ok(Positive::THREE));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the division-by-zero error"]
    pub fn checked_div_positive(&self, rhs: &Positive) -> Result<Positive, PositiveError> {
        self.0.checked_div(rhs)
    }

    /// Sums a **non-empty** iterator of `StrictlyPositive` values.
    ///
    /// There is deliberately no `std::iter::Sum` impl: the sum of an empty
    /// iterator is zero, which this type cannot hold.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the iterator is empty
    /// or the total overflows `Decimal`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// let values = [StrictlyPositive::ONE, StrictlyPositive::TWO];
    /// assert_eq!(StrictlyPositive::checked_sum(values), Ok(StrictlyPositive::THREE));
    /// assert!(StrictlyPositive::checked_sum(Vec::<StrictlyPositive>::new()).is_err());
    /// ```
    #[must_use = "checked aggregation returns a Result; ignoring it silences the error"]
    pub fn checked_sum<I, T>(iter: I) -> Result<StrictlyPositive, PositiveError>
    where
        I: IntoIterator<Item = T>,
        T: Borrow<StrictlyPositive>,
    {
        let mut iter = iter.into_iter();
        let first = iter.next().ok_or_else(|| {
            PositiveError::arithmetic_error("sum", "an empty iterator has no strictly positive sum")
        })?;
        let mut total = first.borrow().to_dec();
        for value in iter {
            total = dec_add(total, value.borrow().to_dec(), "sum")?;
        }
        StrictlyPositive::new_decimal(total)
    }

    // -----------------------------------------------------------------------
    // Mathematical functions
    // -----------------------------------------------------------------------

    /// Square root.
    ///
    /// # Panics
    ///
    /// Panics when the root underflows to zero, which cannot happen for any
    /// representable input in practice. Use [`Self::checked_sqrt`] for the
    /// non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::new_decimal(dec!(16))?.sqrt(), StrictlyPositive::FOUR);
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[must_use]
    pub fn sqrt(&self) -> Self {
        strict_unwrap_or_panic(self.checked_sqrt(), "sqrt")
    }

    /// Square root, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the root cannot be
    /// computed and [`PositiveError::OutOfBounds`] when it underflows to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::FOUR.checked_sqrt(), Ok(StrictlyPositive::TWO));
    /// ```
    #[inline]
    #[must_use = "checked mathematics returns a Result; ignoring it silences the error"]
    pub fn checked_sqrt(&self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_sqrt())
    }

    /// Natural logarithm, as a `Decimal` (it is negative below one).
    ///
    /// Infallible: the only point outside the logarithm's domain is zero,
    /// which this type cannot hold.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal::Decimal;
    ///
    /// assert_eq!(StrictlyPositive::ONE.ln(), Decimal::ZERO);
    /// ```
    #[inline]
    #[must_use]
    pub fn ln(&self) -> Decimal {
        self.0.ln()
    }

    /// Base-10 logarithm, as a `Decimal` (it is negative below one).
    ///
    /// Infallible, for the same reason as [`Self::ln`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::HUNDRED.log10(), dec!(2));
    /// ```
    #[inline]
    #[must_use]
    pub fn log10(&self) -> Decimal {
        self.0.log10()
    }

    /// Computes e^x.
    ///
    /// # Panics
    ///
    /// Panics when the result overflows `Decimal`. Use [`Self::checked_exp`]
    /// for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert!(StrictlyPositive::ONE.exp() > StrictlyPositive::TWO);
    /// ```
    #[must_use]
    pub fn exp(&self) -> Self {
        strict_unwrap_or_panic(self.checked_exp(), "exp")
    }

    /// Computes e^x, returning an error instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the result overflows
    /// `Decimal`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert!(StrictlyPositive::ONE.checked_exp().is_ok());
    /// assert!(matches!(
    ///     StrictlyPositive::THOUSAND.checked_exp(),
    ///     Err(PositiveError::ArithmeticError { .. })
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked mathematics returns a Result; ignoring it silences the overflow error"]
    pub fn checked_exp(&self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_exp())
    }

    /// Raises this value to a signed integer power.
    ///
    /// # Panics
    ///
    /// Panics when the power overflows or underflows to zero. Use
    /// [`Self::checked_powi`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TWO.powi(3), StrictlyPositive::EIGHT);
    /// ```
    #[must_use]
    pub fn powi(&self, n: i64) -> Self {
        strict_unwrap_or_panic(self.checked_powi(n), "powi")
    }

    /// Raises this value to a signed integer power, returning an error
    /// instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power overflows
    /// and [`PositiveError::OutOfBounds`] when it underflows to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_powi(3), Ok(StrictlyPositive::EIGHT));
    /// assert!(matches!(
    ///     StrictlyPositive::MIN.checked_powi(2),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_powi(&self, n: i64) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_powi(n))
    }

    /// Raises this value to an unsigned integer power.
    ///
    /// # Panics
    ///
    /// Panics when the power overflows or underflows to zero. Use
    /// [`Self::checked_powu`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TWO.powu(3), StrictlyPositive::EIGHT);
    /// ```
    #[must_use]
    pub fn powu(&self, n: u64) -> Self {
        strict_unwrap_or_panic(self.checked_powu(n), "powu")
    }

    /// Raises this value to an unsigned integer power, returning an error
    /// instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power overflows
    /// and [`PositiveError::OutOfBounds`] when it underflows to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_powu(3), Ok(StrictlyPositive::EIGHT));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_powu(&self, n: u64) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_powu(n))
    }

    /// Raises this value to a [`Positive`] power.
    ///
    /// # Panics
    ///
    /// Panics when the power cannot be computed, overflows, or underflows to
    /// zero. Use [`Self::checked_pow`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.pow(Positive::THREE), StrictlyPositive::EIGHT);
    /// ```
    #[must_use]
    pub fn pow(&self, n: Positive) -> Self {
        strict_unwrap_or_panic(self.checked_pow(n), "pow")
    }

    /// Raises this value to a [`Positive`] power, returning an error instead
    /// of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the power cannot be
    /// computed or overflows, and [`PositiveError::OutOfBounds`] when it
    /// underflows to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{Positive, StrictlyPositive};
    ///
    /// assert_eq!(StrictlyPositive::TWO.checked_pow(Positive::THREE), Ok(StrictlyPositive::EIGHT));
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_pow(&self, n: Positive) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_pow(n))
    }

    /// Rounds down to the nearest integer.
    ///
    /// # Panics
    ///
    /// Panics for every value below one, whose floor is zero. Use
    /// [`Self::checked_floor`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::new_decimal(dec!(1.9))?.floor(), StrictlyPositive::ONE);
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[must_use]
    pub fn floor(&self) -> Self {
        strict_unwrap_or_panic(self.checked_floor(), "floor")
    }

    /// Rounds down to the nearest integer, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the value is below one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    /// use rust_decimal_macros::dec;
    ///
    /// let small = StrictlyPositive::new_decimal(dec!(0.3))?;
    /// assert!(matches!(small.checked_floor(), Err(PositiveError::OutOfBounds { .. })));
    /// # Ok::<(), PositiveError>(())
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_floor(&self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_floor())
    }

    /// Rounds up to the nearest integer. The result is at least one, so this
    /// cannot break the invariant.
    ///
    /// # Panics
    ///
    /// Never panics in practice: the ceiling of a strictly positive
    /// `Decimal` is at least one and never exceeds `Decimal::MAX`. The panic
    /// path exists only so the method shares the checked core. Use
    /// [`Self::checked_ceiling`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::new_decimal(dec!(0.3))?.ceiling(), StrictlyPositive::ONE);
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[must_use]
    pub fn ceiling(&self) -> Self {
        strict_unwrap_or_panic(self.checked_ceiling(), "ceiling")
    }

    /// Rounds up to the nearest integer, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Same as [`Positive::checked_ceiling`]; no strictly positive input is
    /// known to fail.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// let small = StrictlyPositive::new_decimal(dec!(0.3))?;
    /// assert_eq!(small.checked_ceiling(), Ok(StrictlyPositive::ONE));
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the error"]
    pub fn checked_ceiling(&self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_ceiling())
    }

    /// Rounds to the nearest integer.
    ///
    /// # Panics
    ///
    /// Panics for every value that rounds to zero, such as `0.3`. Use
    /// [`Self::checked_round`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// assert_eq!(StrictlyPositive::new_decimal(dec!(1.6))?.round(), StrictlyPositive::TWO);
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[must_use]
    pub fn round(&self) -> Self {
        strict_unwrap_or_panic(self.checked_round(), "round")
    }

    /// Rounds to the nearest integer, returning an error instead of
    /// panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when the value rounds to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    /// use rust_decimal_macros::dec;
    ///
    /// let small = StrictlyPositive::new_decimal(dec!(0.3))?;
    /// assert!(matches!(small.checked_round(), Err(PositiveError::OutOfBounds { .. })));
    /// # Ok::<(), PositiveError>(())
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_round(&self) -> Result<Self, PositiveError> {
        Self::lift(self.0.checked_round())
    }

    /// Rounds to `decimal_places` decimal places, using `Decimal`'s default
    /// (banker's) rounding.
    ///
    /// # Panics
    ///
    /// Panics when `decimal_places` exceeds 28, or when the value rounds to
    /// zero at that scale, such as `0.004` at two places. Use
    /// [`Self::checked_round_to`] for the non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    /// use rust_decimal_macros::dec;
    ///
    /// let value = StrictlyPositive::new_decimal(dec!(1.2345))?;
    /// assert_eq!(value.round_to(2).to_dec(), dec!(1.23));
    /// # Ok::<(), positive::PositiveError>(())
    /// ```
    #[must_use]
    pub fn round_to(&self, decimal_places: u32) -> Self {
        strict_unwrap_or_panic(self.checked_round_to(decimal_places), "round_to")
    }

    /// Rounds to `decimal_places` decimal places, returning an error instead
    /// of panicking.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidPrecision`] when `decimal_places`
    /// exceeds 28, and [`PositiveError::OutOfBounds`] when the value rounds
    /// to zero at that scale.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    /// use rust_decimal_macros::dec;
    ///
    /// let small = StrictlyPositive::new_decimal(dec!(0.004))?;
    /// assert!(matches!(small.checked_round_to(2), Err(PositiveError::OutOfBounds { .. })));
    /// assert_eq!(small.checked_round_to(3).map(|v| v.to_dec()), Ok(dec!(0.004)));
    /// # Ok::<(), PositiveError>(())
    /// ```
    #[inline]
    #[must_use = "checked arithmetic returns a Result; ignoring it silences the invariant error"]
    pub fn checked_round_to(&self, decimal_places: u32) -> Result<Self, PositiveError> {
        let decimal_places = validate_precision(decimal_places)?;
        Self::lift(self.0.checked_round_to(decimal_places))
    }

    /// Returns the larger of two values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::ONE.max(StrictlyPositive::TWO), StrictlyPositive::TWO);
    /// ```
    #[inline]
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self >= other { self } else { other }
    }

    /// Returns the smaller of two values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// assert_eq!(StrictlyPositive::ONE.min(StrictlyPositive::TWO), StrictlyPositive::ONE);
    /// ```
    #[inline]
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        if self <= other { self } else { other }
    }

    /// Clamps the value to `[min, max]`.
    ///
    /// # Panics
    ///
    /// Panics when `min > max`. Use [`Self::checked_clamp`] for the
    /// non-panicking form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::StrictlyPositive;
    ///
    /// let clamped = StrictlyPositive::TEN.clamp(StrictlyPositive::ONE, StrictlyPositive::FIVE);
    /// assert_eq!(clamped, StrictlyPositive::FIVE);
    /// ```
    #[must_use]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        match self.checked_clamp(min, max) {
            Ok(clamped) => clamped,
            Err(_) => strict_inverted_range_panic(),
        }
    }

    /// Clamps the value to `[min, max]`, returning an error instead of
    /// panicking on an inverted range.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `min > max`. As with
    /// [`Positive::checked_clamp`], `value` is the requested `min` and the
    /// bounds describe the range it had to fall within.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::{PositiveError, StrictlyPositive};
    ///
    /// assert!(matches!(
    ///     StrictlyPositive::TWO.checked_clamp(StrictlyPositive::FIVE, StrictlyPositive::ONE),
    ///     Err(PositiveError::OutOfBounds { .. })
    /// ));
    /// ```
    #[must_use = "checked clamping returns a Result; ignoring it silences the inverted-range error"]
    pub fn checked_clamp(self, min: Self, max: Self) -> Result<Self, PositiveError> {
        if min > max {
            return Err(PositiveError::out_of_bounds(
                min.to_dec(),
                StrictlyPositive::MIN.to_dec(),
                max.to_dec(),
            ));
        }
        Ok(if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        })
    }
}

// ===========================================================================
// Formatting
// ===========================================================================

impl fmt::Display for StrictlyPositive {
    /// Renders the value exactly as [`Positive`] does, honouring a precision
    /// such as `{:.2}`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for StrictlyPositive {
    /// Renders the normalised decimal, as [`Positive`]'s `Debug` does.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

// ===========================================================================
// Serde
// ===========================================================================

impl Serialize for StrictlyPositive {
    /// Serialises the exact decimal string, the same wire format as
    /// [`Positive`].
    ///
    /// # Errors
    ///
    /// Propagates any error from the serializer.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for StrictlyPositive {
    /// Deserialises with the same rules as [`Positive`] (exact decimal string,
    /// or a legacy number), then rejects zero.
    ///
    /// # Errors
    ///
    /// Fails when the input is not a valid decimal, or is zero or negative. A
    /// negative input is rejected by [`Positive`]'s deserializer, so its
    /// message reports `Positive`'s bounds (minimum `0`) rather than this
    /// type's.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Positive::deserialize(deserializer)?;
        StrictlyPositive::from_positive_checked(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "utoipa")]
impl utoipa::PartialSchema for StrictlyPositive {
    /// A string holding the exact decimal, like [`Positive`]'s schema. The
    /// strict lower bound is stated in the description: JSON Schema's
    /// `exclusiveMinimum` applies to numbers and is ignored for strings.
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .description(Some(
                "A decimal value that is guaranteed to be strictly greater than zero, \
                 serialised as an exact decimal string.",
            ))
            .into()
    }
}

#[cfg(feature = "utoipa")]
impl utoipa::ToSchema for StrictlyPositive {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("StrictlyPositive")
    }
}

// ===========================================================================
// Conversions
// ===========================================================================

impl FromStr for StrictlyPositive {
    type Err = PositiveError;

    /// Parses a `StrictlyPositive` from its decimal text representation.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::InvalidValue`] when `s` is not a decimal and
    /// [`PositiveError::OutOfBounds`] when it is zero or negative.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::lift(Positive::from_str(s))
    }
}

impl From<StrictlyPositive> for Positive {
    #[inline]
    fn from(value: StrictlyPositive) -> Self {
        value.0
    }
}

impl From<StrictlyPositive> for Decimal {
    #[inline]
    fn from(value: StrictlyPositive) -> Self {
        value.to_dec()
    }
}

impl From<StrictlyPositive> for f64 {
    /// Infallible, but lossy beyond `f64`'s ~15 significant digits.
    #[inline]
    fn from(value: StrictlyPositive) -> Self {
        value.to_f64()
    }
}

impl AsRef<Positive> for StrictlyPositive {
    #[inline]
    fn as_ref(&self) -> &Positive {
        &self.0
    }
}

impl TryFrom<Positive> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero.
    #[inline]
    fn try_from(value: Positive) -> Result<Self, Self::Error> {
        StrictlyPositive::from_positive_checked(value)
    }
}

impl TryFrom<Decimal> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero or
    /// negative.
    #[inline]
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        StrictlyPositive::new_decimal(value)
    }
}

impl TryFrom<&Decimal> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero or
    /// negative.
    #[inline]
    fn try_from(value: &Decimal) -> Result<Self, Self::Error> {
        StrictlyPositive::new_decimal(*value)
    }
}

impl TryFrom<f64> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Same as [`StrictlyPositive::new`].
    #[inline]
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        StrictlyPositive::new(value)
    }
}

impl TryFrom<i64> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero or
    /// negative.
    #[inline]
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        StrictlyPositive::new_decimal(Decimal::from(value))
    }
}

impl TryFrom<u64> for StrictlyPositive {
    type Error = PositiveError;

    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero.
    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        StrictlyPositive::new_decimal(Decimal::from(value))
    }
}

impl TryFrom<usize> for StrictlyPositive {
    type Error = PositiveError;

    /// Converts exactly, without going through `f64`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::OutOfBounds`] when `value` is zero.
    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        StrictlyPositive::new_decimal(Decimal::from(value))
    }
}

impl TryFrom<StrictlyPositive> for u64 {
    type Error = PositiveError;

    /// Truncates toward zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// exceeds `u64::MAX`.
    #[inline]
    fn try_from(value: StrictlyPositive) -> Result<Self, Self::Error> {
        u64::try_from(value.0)
    }
}

impl TryFrom<StrictlyPositive> for i64 {
    type Error = PositiveError;

    /// Truncates toward zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// exceeds `i64::MAX`.
    #[inline]
    fn try_from(value: StrictlyPositive) -> Result<Self, Self::Error> {
        i64::try_from(value.0)
    }
}

impl TryFrom<StrictlyPositive> for usize {
    type Error = PositiveError;

    /// Truncates toward zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ConversionError`] when the truncated value
    /// exceeds `usize::MAX`.
    #[inline]
    fn try_from(value: StrictlyPositive) -> Result<Self, Self::Error> {
        usize::try_from(value.0)
    }
}

// ===========================================================================
// Comparisons
// ===========================================================================
//
// All exact, and all delegate to `Positive`'s impls so the two types agree.

impl PartialEq<Positive> for StrictlyPositive {
    #[inline]
    fn eq(&self, other: &Positive) -> bool {
        self.0 == *other
    }
}

impl PartialEq<StrictlyPositive> for Positive {
    #[inline]
    fn eq(&self, other: &StrictlyPositive) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Positive> for StrictlyPositive {
    #[inline]
    fn partial_cmp(&self, other: &Positive) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<StrictlyPositive> for Positive {
    #[inline]
    fn partial_cmp(&self, other: &StrictlyPositive) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialEq<Decimal> for StrictlyPositive {
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        self.0 == *other
    }
}

impl PartialEq<StrictlyPositive> for Decimal {
    #[inline]
    fn eq(&self, other: &StrictlyPositive) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Decimal> for StrictlyPositive {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<StrictlyPositive> for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &StrictlyPositive) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialEq<f64> for StrictlyPositive {
    /// Exact comparison; see [`Positive`]'s `PartialEq<f64>`.
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<StrictlyPositive> for f64 {
    #[inline]
    fn eq(&self, other: &StrictlyPositive) -> bool {
        *self == other.0
    }
}

impl PartialOrd<f64> for StrictlyPositive {
    /// Exact comparison; `NaN` is unordered.
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<StrictlyPositive> for f64 {
    #[inline]
    fn partial_cmp(&self, other: &StrictlyPositive) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl AbsDiffEq for StrictlyPositive {
    type Epsilon = Decimal;

    fn default_epsilon() -> Self::Epsilon {
        Positive::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
    }
}

impl RelativeEq for StrictlyPositive {
    fn default_max_relative() -> Self::Epsilon {
        Positive::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.0.relative_eq(&other.0, epsilon, max_relative)
    }
}

// ===========================================================================
// Operators
// ===========================================================================
//
// Operators whose result is a `StrictlyPositive` are thin wrappers over the
// matching `checked_*` method. Operators whose result is a `Positive` delegate
// to `Positive`'s own operator, so they share its panics exactly.

impl Add for StrictlyPositive {
    type Output = StrictlyPositive;
    /// # Panics
    ///
    /// Panics on overflow. See [`StrictlyPositive::checked_add`].
    #[inline]
    fn add(self, rhs: StrictlyPositive) -> StrictlyPositive {
        strict_unwrap_or_panic(self.checked_add(&rhs), "add")
    }
}

impl Add<Positive> for StrictlyPositive {
    type Output = StrictlyPositive;
    /// # Panics
    ///
    /// Panics on overflow. See [`StrictlyPositive::checked_add_positive`].
    #[inline]
    fn add(self, rhs: Positive) -> StrictlyPositive {
        strict_unwrap_or_panic(self.checked_add_positive(&rhs), "add")
    }
}

impl Add<StrictlyPositive> for Positive {
    type Output = StrictlyPositive;
    /// # Panics
    ///
    /// Panics on overflow. See [`StrictlyPositive::checked_add_positive`].
    #[inline]
    fn add(self, rhs: StrictlyPositive) -> StrictlyPositive {
        strict_unwrap_or_panic(rhs.checked_add_positive(&self), "add")
    }
}

impl AddAssign for StrictlyPositive {
    /// # Panics
    ///
    /// Panics on overflow. See [`StrictlyPositive::checked_add`].
    #[inline]
    fn add_assign(&mut self, rhs: StrictlyPositive) {
        *self = strict_unwrap_or_panic(self.checked_add(&rhs), "add_assign");
    }
}

impl AddAssign<Positive> for StrictlyPositive {
    /// # Panics
    ///
    /// Panics on overflow. See [`StrictlyPositive::checked_add_positive`].
    #[inline]
    fn add_assign(&mut self, rhs: Positive) {
        *self = strict_unwrap_or_panic(self.checked_add_positive(&rhs), "add_assign");
    }
}

impl Sub for StrictlyPositive {
    type Output = Positive;
    /// Returns a [`Positive`], because the difference may be zero.
    ///
    /// # Panics
    ///
    /// Same as `Positive - Positive`: panics when the difference is negative.
    /// See [`Positive::checked_sub`], or [`StrictlyPositive::checked_sub`] to
    /// require a strictly positive difference.
    #[inline]
    fn sub(self, rhs: StrictlyPositive) -> Positive {
        unwrap_or_panic(self.0.checked_sub(&rhs.0), "sub")
    }
}

impl Mul for StrictlyPositive {
    type Output = StrictlyPositive;
    /// # Panics
    ///
    /// Panics on overflow, or when the product underflows to zero. See
    /// [`StrictlyPositive::checked_mul`].
    #[inline]
    fn mul(self, rhs: StrictlyPositive) -> StrictlyPositive {
        strict_unwrap_or_panic(self.checked_mul(&rhs), "mul")
    }
}

impl Mul<Positive> for StrictlyPositive {
    type Output = Positive;
    /// # Panics
    ///
    /// Same as `Positive * Positive`. See
    /// [`StrictlyPositive::checked_mul_positive`].
    #[inline]
    fn mul(self, rhs: Positive) -> Positive {
        self.0 * rhs
    }
}

impl Mul<StrictlyPositive> for Positive {
    type Output = Positive;
    /// # Panics
    ///
    /// Same as `Positive * Positive`. See [`Positive::checked_mul`].
    #[inline]
    fn mul(self, rhs: StrictlyPositive) -> Positive {
        self * rhs.0
    }
}

impl MulAssign for StrictlyPositive {
    /// # Panics
    ///
    /// Panics on overflow, or when the product underflows to zero. See
    /// [`StrictlyPositive::checked_mul`].
    #[inline]
    fn mul_assign(&mut self, rhs: StrictlyPositive) {
        *self = strict_unwrap_or_panic(self.checked_mul(&rhs), "mul_assign");
    }
}

impl Div for StrictlyPositive {
    type Output = StrictlyPositive;
    /// Rounds with [`crate::DIV_ROUNDING_STRATEGY`] (banker's rounding).
    ///
    /// # Panics
    ///
    /// Panics on overflow, or when the quotient underflows to zero. See
    /// [`StrictlyPositive::checked_div`].
    #[inline]
    fn div(self, rhs: StrictlyPositive) -> StrictlyPositive {
        strict_unwrap_or_panic(self.checked_div(&rhs), "div")
    }
}

impl Div<Positive> for StrictlyPositive {
    type Output = Positive;
    /// Rounds with [`crate::DIV_ROUNDING_STRATEGY`] (banker's rounding).
    ///
    /// # Panics
    ///
    /// Same as `Positive / Positive`, including on a zero divisor. See
    /// [`StrictlyPositive::checked_div_positive`].
    #[inline]
    fn div(self, rhs: Positive) -> Positive {
        self.0 / rhs
    }
}

impl Div<StrictlyPositive> for Positive {
    type Output = Positive;
    /// Rounds with [`crate::DIV_ROUNDING_STRATEGY`] (banker's rounding). The
    /// divisor is never zero.
    ///
    /// # Panics
    ///
    /// Same as `Positive / Positive`. See [`Positive::checked_div`].
    #[inline]
    fn div(self, rhs: StrictlyPositive) -> Positive {
        self / rhs.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strictly_positive_repr_is_transparent_over_decimal() {
        assert_eq!(
            std::mem::size_of::<StrictlyPositive>(),
            std::mem::size_of::<Decimal>()
        );
    }

    #[test]
    fn test_strictly_positive_constants_are_strictly_positive() {
        let constants = [
            StrictlyPositive::MIN,
            StrictlyPositive::MAX,
            StrictlyPositive::ONE,
            StrictlyPositive::TWO,
            StrictlyPositive::THREE,
            StrictlyPositive::FOUR,
            StrictlyPositive::FIVE,
            StrictlyPositive::SIX,
            StrictlyPositive::SEVEN,
            StrictlyPositive::EIGHT,
            StrictlyPositive::NINE,
            StrictlyPositive::TEN,
            StrictlyPositive::HUNDRED,
            StrictlyPositive::THOUSAND,
            StrictlyPositive::PI,
            StrictlyPositive::E,
        ];
        for constant in constants {
            assert!(constant.to_dec() > Decimal::ZERO);
            assert_eq!(
                StrictlyPositive::new_decimal(constant.to_dec()),
                Ok(constant)
            );
        }
    }

    #[test]
    fn test_strictly_positive_min_is_smallest_decimal_step() {
        assert_eq!(StrictlyPositive::MIN.to_dec(), Decimal::new(1, 28));
    }

    #[test]
    fn test_strictly_positive_lift_relabels_bounds() {
        let err = StrictlyPositive::lift(Positive::new_decimal(dec!(-1)));
        assert_eq!(
            err,
            Err(PositiveError::OutOfBounds {
                value: dec!(-1),
                min: Decimal::new(1, 28),
                max: Decimal::MAX,
            })
        );
    }
}
