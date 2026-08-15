/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! # Positive
//!
//! A type-safe wrapper for guaranteed positive decimal values in Rust.
//!
//! ## Overview
//!
//! `Positive` is a Rust library that provides a type-safe wrapper around `Decimal` values,
//! ensuring that the contained value is always positive. By default, values are non-negative
//! (>= 0). With the `non-zero` feature enabled, values must be strictly positive (> 0).
//! This is particularly useful in financial applications where negative values would be
//! invalid or meaningless, such as prices, quantities, volatilities, and other positive metrics.
//!
//! ## Features
//!
//! - **Type Safety**: Compile-time and runtime guarantees that values are positive
//! - **Non-Zero Mode**: Optional `non-zero` feature flag to reject zero values (strictly > 0)
//! - **Decimal Precision**: Built on [`rust_decimal`](https://crates.io/crates/rust_decimal) for accurate financial calculations
//! - **Rich API**: Comprehensive arithmetic operations, conversions, and mathematical utilities
//! - **Predefined Constants**: Common numeric values (0-10, multiples of 5/100/1000, PI, E, etc.)
//! - **Convenient Macros**: `pos!`, `pos_or_panic!`, `spos!` for easy value creation
//! - **Prelude Module**: Simple imports with `use positive::prelude::*;`
//! - **Serde Support**: Lossless serialisation as exact decimal strings, for JSON and binary formats alike
//! - **Approx Support**: Approximate equality comparisons for floating-point tolerance
//! - **Checked Operations**: Safe arithmetic operations that return `Result` instead of panicking
//! - **Optional utoipa Integration**: OpenAPI schema generation support via feature flag
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! positive = "0.6"
//! ```
//!
//! To require strictly positive values (excluding zero):
//!
//! ```toml
//! [dependencies]
//! positive = { version = "0.6", features = ["non-zero"] }
//! ```
//!
//! To enable OpenAPI schema support:
//!
//! ```toml
//! [dependencies]
//! positive = { version = "0.6", features = ["utoipa"] }
//! ```
//!
//! ## Quick Start
//!
//! The recommended pattern is fallible construction and checked arithmetic,
//! propagating [`PositiveError`] with `?`. Nothing here can panic:
//!
//! ```rust
//! use positive::prelude::*;
//!
//! fn order_total() -> Result<Positive, PositiveError> {
//!     let price = Positive::new(100.50)?;
//!     let quantity = Positive::new(10.0)?;
//!     let discount = Positive::new(5.0)?;
//!
//!     let subtotal = price.checked_mul(&quantity)?;
//!     let after_discount = subtotal.checked_sub(&discount)?;
//!
//!     // Constants are ready-made and cannot fail
//!     let tax_rate = FIVE.checked_div(&HUNDRED)?;   // 5%
//!     let tax = after_discount.checked_mul(&tax_rate)?;
//!
//!     after_discount.checked_add(&tax)
//! }
//!
//! assert!(order_total().is_ok());
//! ```
//!
//! The operators (`+`, `-`, `*`, `/`) are available too and read more
//! naturally, at the cost of panicking on overflow or on a result that would
//! break the invariant. Every one of them has a `checked_` counterpart, listed
//! in its `# Panics` section, so the panicking form is always an opt-in:
//!
//! ```rust
//! use positive::prelude::*;
//!
//! # fn main() -> Result<(), PositiveError> {
//! let price = Positive::new(100.50)?;
//! let quantity = Positive::new(10.0)?;
//! let total = price * quantity;          // panics on overflow
//! let total = price.checked_mul(&quantity)?;  // returns Err instead
//! # Ok(())
//! # }
//! ```
//!
//! ### A note on the examples below
//!
//! The remaining examples use [`pos_or_panic!`] for brevity, so each one fits
//! in a line or two. That macro panics on invalid input and is intended for
//! tests, examples and constant literals — **not** for production paths that
//! handle external input. There, use [`Positive::new`], [`pos!`] or [`spos!`]
//! and handle the failure.
//!
//! ## API Overview
//!
//! ### Creation
//!
//! ```rust
//! use positive::{Positive, pos, pos_or_panic, spos};
//! use rust_decimal::Decimal;
//!
//! // From f64
//! let p = Positive::new(5.0).unwrap();
//!
//! // From Decimal
//! let p = Positive::new_decimal(Decimal::ONE).unwrap();
//!
//! // Using macros
//! let p = pos!(5.0);           // Returns Result<Positive, PositiveError>
//! let p = pos_or_panic!(5.0);  // Panics on invalid input
//! let p = spos!(5.0);          // Returns Option<Positive>
//! ```
//!
//! ### Constants
//!
//! The library provides many predefined constants accessible via `Positive::CONSTANT`
//! or directly from the `constants` module:
//!
//! ```rust
//! use positive::Positive;
//! use positive::constants::*;
//!
//! // Integer constants (1-10)
//! let one = Positive::ONE;         // 1
//! let two = Positive::TWO;         // 2
//! let ten = Positive::TEN;         // 10
//!
//! // Multiples of 5 (15-95)
//! let fifteen = FIFTEEN;           // 15
//! let fifty = FIFTY;               // 50
//!
//! // Multiples of 100 (100-900)
//! let hundred = Positive::HUNDRED; // 100
//! let five_hundred = FIVE_HUNDRED; // 500
//!
//! // Multiples of 1000 (1000-10000)
//! let thousand = Positive::THOUSAND; // 1000
//! let ten_thousand = TEN_THOUSAND;   // 10000
//!
//! // Mathematical constants
//! let pi = Positive::PI;           // π (3.14159...)
//! let e = Positive::E;             // e (2.71828...)
//!
//! // Special values
//! let epsilon = EPSILON;           // Small tolerance for comparisons
//! let max = Positive::MAX;         // Largest representable value (Decimal::MAX)
//! ```
//!
//! ### Conversions
//!
//! ```rust
//! use positive::pos_or_panic;
//!
//! let p = pos_or_panic!(5.5);
//!
//! let f: f64 = p.to_f64();                  // Infallible, lossy beyond ~15 digits
//! let d = p.to_dec();                       // To Decimal, exact
//!
//! // Integer conversions are fallible and truncate toward zero
//! let i: Result<i64, _> = i64::try_from(p);
//! let u: Result<u64, _> = u64::try_from(p);
//! let n: Result<usize, _> = usize::try_from(p);
//! let maybe: Option<u64> = p.to_u64_checked();
//! ```
//!
//! ### Arithmetic Operations
//!
//! ```rust
//! use positive::pos_or_panic;
//!
//! let a = pos_or_panic!(10.0);
//! let b = pos_or_panic!(3.0);
//!
//! // Standard operations
//! let sum = a + b;        // Addition
//! let diff = a - b;       // Subtraction (panics if result < 0)
//! let prod = a * b;       // Multiplication
//! let quot = a / b;       // Division
//!
//! // Safe operations
//! let safe_diff = a.checked_sub(&b);    // Returns Result
//! let safe_quot = a.checked_div(&b);    // Returns Result (handles div by zero)
//! ```
//!
//! ### Mathematical Functions
//!
//! ```rust
//! use positive::pos_or_panic;
//!
//! let p = pos_or_panic!(16.0);
//!
//! let sqrt = p.sqrt();           // Square root
//! let ln = p.ln();               // Natural logarithm
//! let log10 = p.log10();         // Base-10 logarithm
//! let exp = p.exp();             // Exponential (e^x)
//! let pow = p.pow(pos_or_panic!(2.0));    // Power with Positive exponent
//! let powi = p.powi(2);          // Integer power
//! let floor = p.floor();         // Floor
//! let ceil = p.ceiling();        // Ceiling
//! let round = p.round();         // Round to nearest integer
//! let round2 = p.round_to(2);    // Round to 2 decimal places
//! ```
//!
//! ### Utility Methods
//!
//! ```rust
//! use positive::pos_or_panic;
//!
//! use rust_decimal_macros::dec;
//! let p = pos_or_panic!(5.0);
//!
//! let is_zero = p.is_zero();                      // Check if zero
//! let is_mult = p.is_multiple_of_dec(dec!(2));    // Check if multiple of value
//! let clamped = p.clamp(pos_or_panic!(1.0), pos_or_panic!(10.0));   // Clamp between bounds
//! let min_val = p.min(pos_or_panic!(3.0));                 // Minimum of two values
//! let max_val = p.max(pos_or_panic!(3.0));                 // Maximum of two values
//! let formatted = p.format_fixed_places(2);       // Format with fixed decimals
//! ```
//!
//! ## Error Handling
//!
//! The library provides `PositiveError` for comprehensive error handling:
//!
//! ```rust
//! use positive::{Positive, PositiveError};
//!
//! fn example() -> Result<Positive, PositiveError> {
//!     let value = Positive::new(-5.0)?;  // Returns Err(OutOfBounds)
//!     Ok(value)
//! }
//! ```
//!
//! `PositiveError` has exactly five variants, and that set is stable across
//! minor versions. There is no catch-all, so callers can match exhaustively
//! without a wildcard arm:
//!
//! - `InvalidValue` - Input that is not a decimal at all (`NaN`, `±inf`, unparsable text)
//! - `ArithmeticError` - Overflow, division by zero, or a result breaking the invariant
//! - `ConversionError` - A valid value not representable in the destination type
//! - `OutOfBounds` - A well-formed decimal outside the permitted range
//! - `InvalidPrecision` - A decimal precision outside the range `Decimal` supports
//!
//! `OutOfBounds` carries exact `Decimal` values for the offending input and
//! both bounds, so no precision is lost in the diagnostic. Under the
//! `non-zero` feature the reported minimum is `1e-28`, the smallest strictly
//! positive `Decimal`.
//!
//! Parsing follows the same contract — `FromStr` fails with a `PositiveError`
//! that preserves the offending input:
//!
//! ```rust
//! use positive::{Positive, PositiveError};
//! use std::str::FromStr;
//!
//! let err = Positive::from_str("not a number").unwrap_err();
//! assert!(matches!(err, PositiveError::InvalidValue { .. }));
//! ```
//!
//! ## Serialization
//!
//! `Positive` serialises as the **exact decimal string**, so every value the
//! type can hold round-trips without losing a digit:
//!
//! ```rust
//! use positive::{Positive, pos_or_panic};
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! let p = pos_or_panic!(42.5);
//! let json = serde_json::to_string(&p).unwrap();      // "\"42.5\""
//! let parsed: Positive = serde_json::from_str(&json).unwrap();
//! assert_eq!(parsed, p);
//!
//! // Full 28-digit precision survives the round trip
//! let exact = Decimal::from_str("0.1234567890123456789012345678").unwrap();
//! let value = Positive::new_decimal(exact).unwrap();
//! let json = serde_json::to_string(&value).unwrap();
//! let back: Positive = serde_json::from_str(&json).unwrap();
//! assert_eq!(back.to_dec(), exact);
//! ```
//!
//! ### Precision guarantees
//!
//! - **Representation**: a JSON string holding the exact decimal, e.g.
//!   `"42.5"`, `"79228162514264337593543950335"`.
//! - **Lossless for every representable value**, including 28-digit fractions,
//!   integers above `i64::MAX`, and `Positive::MAX`.
//! - **Validation on the way in**: the positivity invariant is enforced on
//!   deserialisation, so the `non-zero` feature rejects zero there too.
//! - **Non-self-describing formats** (bincode, postcard) are supported: the
//!   implementation asks for a string rather than relying on
//!   `deserialize_any`, which such formats cannot provide.
//! - **Backward compatibility**: plain JSON numbers written by 0.5.x still
//!   deserialise. They are lossy by construction — that precision was gone
//!   before the bytes were written — so re-serialising upgrades them to the
//!   exact form.
//!
//! A JSON *number* cannot carry this precision: nearly every consumer parses
//! one into an `f64`, which holds about 15 significant digits. That is why the
//! wire format is a string, and why `rust_decimal` itself uses one.
//!
//! ## Use Cases
//!
//! - **Financial Applications**: Prices, quantities, fees, rates
//! - **Scientific Computing**: Physical quantities that cannot be negative
//! - **Game Development**: Health points, distances, timers
//! - **Data Validation**: Ensuring input values meet positivity constraints
//!
//! ## Safety
//!
//! This crate contains no `unsafe` code, enforced at compile time by
//! `#![forbid(unsafe_code)]`. Earlier versions exposed a public
//! `Positive::new_unchecked`, an `unsafe fn` that performed no unsafe
//! operation: it moved a logical precondition onto the caller without making
//! a violation Rust undefined behaviour. It has been removed. Every public
//! path that yields a `Positive` validates the invariant.
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//!

#![forbid(unsafe_code)]

pub mod constants;
pub mod error;
#[macro_use]
pub mod macros;
mod positive;
pub mod prelude;
mod tests;
pub use error::{PositiveError, PositiveResult};
pub use positive::{DIV_ROUNDING_STRATEGY, Positive, is_positive, is_valid_positive_value};

/// Re-export rust_decimal for convenience.
pub use rust_decimal::Decimal;
