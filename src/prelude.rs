/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used items from the library,
//! allowing users to import everything they need with a single use statement:
//!
//! ```rust
//! use positive::prelude::*;
//! ```
//!
//! This includes:
//! - The `Positive` type and its associated macros
//! - The `StrictlyPositive` type and its macros (`strict_pos!`, `strict_pos_or_panic!`)
//! - Error types for handling failures
//! - The `Decimal` type from `rust_decimal`
//! - All predefined constants

pub use crate::constants::*;
pub use crate::error::{PositiveError, PositiveResult};
pub use crate::{
    Positive, StrictlyPositive, is_positive, pos, pos_or_panic, spos, strict_pos,
    strict_pos_or_panic,
};
pub use rust_decimal::Decimal;
