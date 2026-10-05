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

/// Macro for creating a `StrictlyPositive` value (`> 0`) from an `f64`
/// expression.
///
/// Returns `Ok(StrictlyPositive)` when the value is strictly greater than
/// zero, otherwise `Err(PositiveError)`.
///
/// # Example
///
/// ```rust
/// use positive::strict_pos;
///
/// assert!(strict_pos!(5.0).is_ok());
/// assert!(strict_pos!(0.0).is_err());
/// assert!(strict_pos!(-5.0).is_err());
/// ```
#[macro_export]
macro_rules! strict_pos {
    ($val:expr) => {
        $crate::StrictlyPositive::new($val)
    };
}

/// Macro for creating a `StrictlyPositive` value that panics on invalid
/// input.
///
/// Intended for tests, examples and literals, like [`pos_or_panic!`]. Use
/// [`strict_pos!`] or `StrictlyPositive::new` on production paths.
///
/// # Panics
///
/// Panics when the value is zero, negative, `NaN`, infinite or not
/// representable as a `Decimal`.
///
/// # Example
///
/// ```rust
/// use positive::strict_pos_or_panic;
///
/// let value = strict_pos_or_panic!(5.0);
/// assert_eq!(value.to_f64(), 5.0);
/// ```
#[macro_export]
macro_rules! strict_pos_or_panic {
    ($val:expr) => {
        // Panicking on invalid input is this macro's entire contract, as for
        // `pos_or_panic!`; it is sanctioned for tests, examples and literals.
        $crate::StrictlyPositive::new($val).expect("Failed to create StrictlyPositive value") // scan-banned: allow
    };
}
