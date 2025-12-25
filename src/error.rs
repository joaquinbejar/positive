/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/

//! Error types for the Positive decimal type.
//!
//! This module provides error handling for operations involving positive decimal values,
//! including validation, arithmetic operations, conversions, and precision issues.

use thiserror::Error;

/// Represents errors that can occur during positive decimal operations.
///
/// This enum provides a structured way to handle various error conditions that may arise
/// when working with positive decimal values, including validation, arithmetic operations,
/// conversions, and precision issues.
///
/// # Variants
///
/// * `InvalidValue` - Value cannot be represented as a valid positive decimal
/// * `ArithmeticError` - Error during mathematical operations
/// * `ConversionError` - Error when converting between types
/// * `OutOfBounds` - Value exceeds defined limits
/// * `InvalidPrecision` - Invalid decimal precision settings
/// * `Other` - Catch-all for other errors
#[derive(Error, Debug)]
pub enum PositiveError {
    /// Error when attempting to create a positive decimal from an invalid value.
    ///
    /// Occurs when a value cannot be properly represented as a positive decimal,
    /// such as when it's NaN, infinity, negative, or otherwise unsuitable.
    #[error("Invalid positive value {value}: {reason}")]
    InvalidValue {
        /// The problematic value that caused the error.
        value: f64,
        /// Detailed explanation of why the value is invalid.
        reason: String,
    },

    /// Error when performing decimal arithmetic operations.
    ///
    /// Occurs during mathematical operations such as addition, subtraction,
    /// multiplication, or division when the operation cannot be completed
    /// correctly (e.g., division by zero, overflow, result would be negative).
    #[error("Arithmetic error during {operation}: {reason}")]
    ArithmeticError {
        /// The operation that failed (e.g., "subtraction", "division").
        operation: String,
        /// Detailed explanation of why the operation failed.
        reason: String,
    },

    /// Error when converting between decimal types.
    ///
    /// Occurs when a decimal value cannot be correctly converted from one
    /// representation to another, such as between different precision levels
    /// or between different decimal formats.
    #[error("Failed to convert from {from_type} to {to_type}: {reason}")]
    ConversionError {
        /// The source type being converted from.
        from_type: String,
        /// The destination type being converted to.
        to_type: String,
        /// Detailed explanation of why the conversion failed.
        reason: String,
    },

    /// Error when a decimal value exceeds its bounds.
    ///
    /// Occurs when a decimal value falls outside of acceptable minimum
    /// or maximum values for a specific calculation context.
    #[error("Value {value} is out of bounds (min: {min}, max: {max})")]
    OutOfBounds {
        /// The value that is out of bounds.
        value: f64,
        /// The minimum acceptable value.
        min: f64,
        /// The maximum acceptable value.
        max: f64,
    },

    /// Error when decimal precision is invalid.
    ///
    /// Occurs when an operation specifies or results in an invalid precision
    /// level that cannot be properly handled.
    #[error("Invalid precision {precision}: {reason}")]
    InvalidPrecision {
        /// The problematic precision value.
        precision: i32,
        /// Detailed explanation of why the precision is invalid.
        reason: String,
    },

    /// Catch-all error for other positive decimal errors.
    #[error("Positive error: {0}")]
    Other(String),
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
    /// * `value` - The problematic floating-point value
    /// * `reason` - Explanation of why the value is invalid
    ///
    /// # Returns
    ///
    /// A new `PositiveError::InvalidValue` instance
    #[must_use]
    pub fn invalid_value(value: f64, reason: &str) -> Self {
        PositiveError::InvalidValue {
            value,
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
    /// * `value` - The out-of-bounds floating-point value
    /// * `min` - The lower bound (inclusive) of the valid range
    /// * `max` - The upper bound (inclusive) of the valid range
    ///
    /// # Returns
    ///
    /// A new `PositiveError::OutOfBounds` instance
    #[must_use]
    pub fn out_of_bounds(value: f64, min: f64, max: f64) -> Self {
        PositiveError::OutOfBounds { value, min, max }
    }

    /// Creates a new `InvalidPrecision` error.
    ///
    /// # Arguments
    ///
    /// * `precision` - The problematic precision value
    /// * `reason` - Explanation of why the precision is invalid
    ///
    /// # Returns
    ///
    /// A new `PositiveError::InvalidPrecision` instance
    #[must_use]
    pub fn invalid_precision(precision: i32, reason: &str) -> Self {
        PositiveError::InvalidPrecision {
            precision,
            reason: reason.to_string(),
        }
    }
}

impl From<&str> for PositiveError {
    fn from(s: &str) -> Self {
        PositiveError::Other(s.to_string())
    }
}

impl From<String> for PositiveError {
    fn from(s: String) -> Self {
        PositiveError::Other(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_value_error() {
        let error = PositiveError::invalid_value(-1.0, "Value cannot be negative");
        assert!(matches!(error, PositiveError::InvalidValue { .. }));
        assert!(error.to_string().contains("cannot be negative"));
    }

    #[test]
    fn test_arithmetic_error() {
        let error = PositiveError::arithmetic_error("subtraction", "Result would be negative");
        assert!(matches!(error, PositiveError::ArithmeticError { .. }));
        assert!(error.to_string().contains("would be negative"));
    }

    #[test]
    fn test_conversion_error() {
        let error = PositiveError::conversion_error("f64", "Positive", "Value out of range");
        assert!(matches!(error, PositiveError::ConversionError { .. }));
        assert!(error.to_string().contains("out of range"));
    }

    #[test]
    fn test_out_of_bounds_error() {
        let error = PositiveError::out_of_bounds(-5.0, 0.0, 100.0);
        assert!(matches!(error, PositiveError::OutOfBounds { .. }));
        assert!(error.to_string().contains("-5"));
    }

    #[test]
    fn test_invalid_precision_error() {
        let error = PositiveError::invalid_precision(-1, "Precision must be non-negative");
        assert!(matches!(error, PositiveError::InvalidPrecision { .. }));
        assert!(error.to_string().contains("non-negative"));
    }

    #[test]
    fn test_from_str() {
        let error: PositiveError = "Custom error message".into();
        assert!(matches!(error, PositiveError::Other(_)));
        assert!(error.to_string().contains("Custom error message"));
    }

    #[test]
    fn test_from_string() {
        let error: PositiveError = String::from("Another error").into();
        assert!(matches!(error, PositiveError::Other(_)));
        assert!(error.to_string().contains("Another error"));
    }
}
