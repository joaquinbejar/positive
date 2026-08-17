/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Error types for the Positive decimal type.
//!
//! This module provides error handling for operations involving positive decimal values,
//! including validation, arithmetic operations, conversions, and precision issues.
//!
//! # The stable variant set
//!
//! [`PositiveError`] has exactly five variants and that set is stable across
//! minor versions:
//!
//! | Variant | Raised by |
//! |---|---|
//! | [`PositiveError::InvalidValue`] | input that cannot be interpreted as a decimal at all (`NaN`, `±inf`, unparsable text) |
//! | [`PositiveError::ArithmeticError`] | overflow, division by zero, or a result that breaks the positivity invariant |
//! | [`PositiveError::ConversionError`] | a value that is valid but not representable in the destination type |
//! | [`PositiveError::OutOfBounds`] | a well-formed decimal outside the permitted range |
//! | [`PositiveError::InvalidPrecision`] | a decimal precision outside the range `Decimal` supports |
//!
//! No catch-all variant exists. Every fallible public API in this crate
//! returns one of the five above, so callers can match exhaustively without a
//! wildcard arm.
//!
//! # Exact values, no `f64` projection
//!
//! [`PositiveError::OutOfBounds`] carries `Decimal` values for the offending
//! input and both bounds. Projecting them through `f64` — as previous versions
//! did — silently rounded the very value the caller needed to diagnose, and
//! could not represent the true bounds at all: under the `non-zero` feature the
//! smallest permitted value is `1e-28`, which is far below `f64::MIN_POSITIVE`
//! in decimal terms and is not a binary-representable float.
//!
//! [`PositiveError::InvalidValue`] carries the offending input rendered as a
//! `String`, because the inputs it reports on (`NaN`, `±inf`, arbitrary text)
//! have no `Decimal` representation by definition.
//!
//! # Message style
//!
//! Every `Display` message begins with a lowercase letter and includes the
//! offending input where one exists, so messages compose cleanly when wrapped
//! by a caller's own error type.

use rust_decimal::Decimal;
use thiserror::Error;

/// Represents errors that can occur during positive decimal operations.
///
/// This enum provides a structured way to handle various error conditions that may arise
/// when working with positive decimal values, including validation, arithmetic operations,
/// conversions, and precision issues.
///
/// See the [module documentation](self) for the stable variant set and the
/// rationale behind the field types.
///
/// # Examples
///
/// ```rust
/// use positive::{Positive, PositiveError};
///
/// let err = Positive::new(-1.0).unwrap_err();
/// assert!(matches!(err, PositiveError::OutOfBounds { .. }));
/// assert!(err.to_string().starts_with("value -1 is out of bounds"));
/// ```
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PositiveError {
    /// Error when attempting to create a positive decimal from an invalid value.
    ///
    /// Occurs when a value cannot be interpreted as a decimal at all — `NaN`,
    /// `±inf`, or text that does not parse. Values that *are* well-formed
    /// decimals but fall outside the permitted range produce
    /// [`PositiveError::OutOfBounds`] instead.
    ///
    /// `value` holds the offending input rendered exactly as it was supplied.
    #[error("invalid positive value '{value}': {reason}")]
    InvalidValue {
        /// The problematic input, rendered exactly as supplied.
        value: String,
        /// Detailed explanation of why the value is invalid.
        reason: String,
    },

    /// Error when performing decimal arithmetic operations.
    ///
    /// Occurs during mathematical operations such as addition, subtraction,
    /// multiplication, or division when the operation cannot be completed
    /// correctly (e.g., division by zero, overflow, result would be negative).
    #[error("arithmetic error during {operation}: {reason}")]
    ArithmeticError {
        /// The operation that failed (e.g., "subtraction", "division").
        operation: String,
        /// Detailed explanation of why the operation failed.
        reason: String,
    },

    /// Error when converting between decimal types.
    ///
    /// Occurs when a value is well-formed but cannot be represented in the
    /// destination type — for example a `Decimal` larger than `u64::MAX`, or an
    /// `f64` outside the range `Decimal` can hold.
    #[error("failed to convert from {from_type} to {to_type}: {reason}")]
    ConversionError {
        /// The source type being converted from.
        from_type: String,
        /// The destination type being converted to.
        to_type: String,
        /// Detailed explanation of why the conversion failed.
        reason: String,
    },

    /// Error when a decimal value falls outside the permitted range.
    ///
    /// All three fields are exact `Decimal` values. `min` reflects the active
    /// feature configuration: `0` by default, and `1e-28` — the smallest
    /// strictly positive `Decimal` — under the `non-zero` feature.
    #[error("value {value} is out of bounds (min: {min}, max: {max})")]
    OutOfBounds {
        /// The value that is out of bounds.
        value: Decimal,
        /// The minimum acceptable value, inclusive.
        min: Decimal,
        /// The maximum acceptable value, inclusive.
        max: Decimal,
    },

    /// Error when decimal precision is invalid.
    ///
    /// Occurs when an operation specifies a number of decimal places outside
    /// the range `rust_decimal::Decimal` supports (0 through 28 inclusive).
    #[error("invalid precision {precision}: {reason}")]
    InvalidPrecision {
        /// The problematic precision value, in decimal places.
        precision: u32,
        /// Detailed explanation of why the precision is invalid.
        reason: String,
    },
}

/// A specialized `Result` type for positive decimal operations.
///
/// This type alias provides a convenient shorthand for operations that can result in a
/// `PositiveError`. It helps improve code readability and reduces boilerplate.
///
/// # Type Parameters
///
/// * `T` - The successful result type of the operation
pub type PositiveResult<T> = Result<T, PositiveError>;

impl PositiveError {
    /// Creates a new `InvalidValue` error.
    ///
    /// # Arguments
    ///
    /// * `value` - The problematic input, rendered exactly as supplied
    /// * `reason` - Explanation of why the value is invalid
    ///
    /// # Returns
    ///
    /// A new `PositiveError::InvalidValue` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::PositiveError;
    ///
    /// let err = PositiveError::invalid_value("NaN", "not a number");
    /// assert_eq!(err.to_string(), "invalid positive value 'NaN': not a number");
    /// ```
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn invalid_value(value: &str, reason: &str) -> Self {
        PositiveError::InvalidValue {
            value: value.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `ArithmeticError` error.
    ///
    /// # Arguments
    ///
    /// * `operation` - The name of the operation that failed
    /// * `reason` - Explanation of why the operation failed
    ///
    /// # Returns
    ///
    /// A new `PositiveError::ArithmeticError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::PositiveError;
    ///
    /// let err = PositiveError::arithmetic_error("division", "division by zero");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "arithmetic error during division: division by zero"
    /// );
    /// ```
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn arithmetic_error(operation: &str, reason: &str) -> Self {
        PositiveError::ArithmeticError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `ConversionError` error.
    ///
    /// # Arguments
    ///
    /// * `from_type` - The source type being converted from
    /// * `to_type` - The destination type being converted to
    /// * `reason` - Explanation of why the conversion failed
    ///
    /// # Returns
    ///
    /// A new `PositiveError::ConversionError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::PositiveError;
    ///
    /// let err = PositiveError::conversion_error("Positive", "u64", "value exceeds u64::MAX");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "failed to convert from Positive to u64: value exceeds u64::MAX"
    /// );
    /// ```
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn conversion_error(from_type: &str, to_type: &str, reason: &str) -> Self {
        PositiveError::ConversionError {
            from_type: from_type.to_string(),
            to_type: to_type.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `OutOfBounds` error.
    ///
    /// # Arguments
    ///
    /// * `value` - The out-of-bounds value, exact
    /// * `min` - The lower bound (inclusive) of the valid range
    /// * `max` - The upper bound (inclusive) of the valid range
    ///
    /// # Returns
    ///
    /// A new `PositiveError::OutOfBounds` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::PositiveError;
    /// use rust_decimal::Decimal;
    /// use rust_decimal_macros::dec;
    ///
    /// let err = PositiveError::out_of_bounds(dec!(-5), Decimal::ZERO, Decimal::MAX);
    /// assert!(err.to_string().starts_with("value -5 is out of bounds"));
    /// ```
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn out_of_bounds(value: Decimal, min: Decimal, max: Decimal) -> Self {
        PositiveError::OutOfBounds { value, min, max }
    }

    /// Creates a new `InvalidPrecision` error.
    ///
    /// # Arguments
    ///
    /// * `precision` - The problematic precision, in decimal places
    /// * `reason` - Explanation of why the precision is invalid
    ///
    /// # Returns
    ///
    /// A new `PositiveError::InvalidPrecision` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use positive::PositiveError;
    ///
    /// let err = PositiveError::invalid_precision(29, "decimal supports at most 28 places");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "invalid precision 29: decimal supports at most 28 places"
    /// );
    /// ```
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn invalid_precision(precision: u32, reason: &str) -> Self {
        PositiveError::InvalidPrecision {
            precision,
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_invalid_value_error() {
        let error = PositiveError::invalid_value("-1", "value cannot be negative");
        assert!(matches!(error, PositiveError::InvalidValue { .. }));
        assert_eq!(
            error.to_string(),
            "invalid positive value '-1': value cannot be negative"
        );
    }

    #[test]
    fn test_arithmetic_error() {
        let error = PositiveError::arithmetic_error("subtraction", "result would be negative");
        assert!(matches!(error, PositiveError::ArithmeticError { .. }));
        assert_eq!(
            error.to_string(),
            "arithmetic error during subtraction: result would be negative"
        );
    }

    #[test]
    fn test_conversion_error() {
        let error = PositiveError::conversion_error("f64", "Positive", "value out of range");
        assert!(matches!(error, PositiveError::ConversionError { .. }));
        assert_eq!(
            error.to_string(),
            "failed to convert from f64 to Positive: value out of range"
        );
    }

    #[test]
    fn test_out_of_bounds_error_keeps_exact_decimal() {
        let value = dec!(-0.0000000000000000000000000001);
        let error = PositiveError::out_of_bounds(value, Decimal::ZERO, Decimal::MAX);
        match &error {
            PositiveError::OutOfBounds { value: v, min, max } => {
                assert_eq!(*v, value);
                assert_eq!(*min, Decimal::ZERO);
                assert_eq!(*max, Decimal::MAX);
            }
            other => panic!("expected OutOfBounds, got {other:?}"),
        }
        assert!(
            error
                .to_string()
                .starts_with("value -0.0000000000000000000000000001 is out of bounds")
        );
    }

    #[test]
    fn test_invalid_precision_error() {
        let error = PositiveError::invalid_precision(29, "decimal supports at most 28 places");
        assert!(matches!(error, PositiveError::InvalidPrecision { .. }));
        assert_eq!(
            error.to_string(),
            "invalid precision 29: decimal supports at most 28 places"
        );
    }

    /// Every `Display` message must start lowercase so it composes when a
    /// caller wraps it in their own error type.
    #[test]
    fn test_all_messages_start_lowercase() {
        let errors = [
            PositiveError::invalid_value("NaN", "not a number"),
            PositiveError::arithmetic_error("division", "division by zero"),
            PositiveError::conversion_error("f64", "Positive", "out of range"),
            PositiveError::out_of_bounds(dec!(-1), Decimal::ZERO, Decimal::MAX),
            PositiveError::invalid_precision(29, "too large"),
        ];
        for error in &errors {
            let rendered = error.to_string();
            let first = rendered
                .chars()
                .next()
                .expect("error message must not be empty");
            assert!(
                first.is_lowercase(),
                "message does not start lowercase: {rendered}"
            );
        }
    }

    #[test]
    fn test_error_is_clone_and_eq() {
        let error = PositiveError::arithmetic_error("division", "division by zero");
        assert_eq!(error.clone(), error);
        assert_ne!(
            error,
            PositiveError::arithmetic_error("division", "overflow")
        );
    }
}
