/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Macros for creating `Positive` values.
//!
//! This module provides convenient macros for creating `Positive` values
//! with different error handling strategies.

/// Macro for creating a `Positive` value from the given expression.
///
/// Returns `Ok(Positive)` if the value is valid and non-negative,
/// otherwise returns `Err(PositiveError)`.
///
/// # Example
///
/// ```rust
/// use positive::pos;
///
/// let valid = pos!(5.0);
/// assert!(valid.is_ok());
///
/// let invalid = pos!(-5.0);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! pos {
    ($val:expr) => {
        $crate::Positive::new($val)
    };
}

/// Macro for creating a new `Positive` value that panics on invalid input.
///
/// Use this macro when you are certain the value is valid and want to
/// avoid handling the `Result`. For safer alternatives, use `pos!()` which
/// returns `Result<Positive, PositiveError>`.
///
/// # Panics
///
/// This macro will panic if the provided value cannot be converted to a `Positive` value
/// (e.g., negative numbers or values that cannot be represented as `Decimal`).
///
/// # Example
///
/// ```rust
/// use positive::pos_or_panic;
///
/// let value = pos_or_panic!(5.0);
/// assert_eq!(value.to_f64(), 5.0);
/// ```
#[macro_export]
macro_rules! pos_or_panic {
    ($val:expr) => {
        // Panicking on invalid input is this macro's entire contract, and
        // rules/global_rules.md sanctions it explicitly for tests, examples
        // and literals.
        $crate::Positive::new($val).expect("Failed to create Positive value") // scan-banned: allow
    };
}

/// Macro for creating an optional `Positive` value from the given expression.
///
/// Returns `Some(Positive)` if the value is valid and non-negative,
/// otherwise returns `None`. This is useful when you want to ignore errors.
///
/// # Example
///
/// ```rust
/// use positive::spos;
///
/// let valid = spos!(5.0);
/// assert!(valid.is_some());
///
/// let invalid = spos!(-5.0);
/// assert!(invalid.is_none());
/// ```
#[macro_export]
macro_rules! spos {
    ($val:expr) => {
        $crate::Positive::new($val).ok()
    };
}
