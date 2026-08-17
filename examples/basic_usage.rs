/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Basic usage example for the Positive library.
//!
//! This example demonstrates the fundamental operations with `Positive` values.

use positive::prelude::*;

fn main() {
    println!("=== Basic Positive Usage ===\n");

    // Creating Positive values using different methods
    println!("--- Creating Positive Values ---");

    // Using the pos! macro (returns Result)
    let price = pos!(100.50).expect("Valid positive value");
    println!("Price (pos! macro): {price}");

    // Using pos_or_panic! for known valid values
    let quantity = pos_or_panic!(10.0);
    println!("Quantity (pos_or_panic!): {quantity}");

    // Using spos! for optional values
    let maybe_value = spos!(25.0);
    println!("Optional value (spos!): {maybe_value:?}");

    // Using the constructor
    let rate = Positive::new(0.05).expect("Valid rate");
    println!("Rate (constructor): {rate}");

    // Arithmetic operations
    println!("\n--- Arithmetic Operations ---");

    let a = pos_or_panic!(20.0);
    let b = pos_or_panic!(5.0);

    println!("a = {a}, b = {b}");
    println!("a + b = {}", a + b);
    println!("a - b = {}", a - b);
    println!("a * b = {}", a * b);
    println!("a / b = {}", a / b);

    // Safe operations
    println!("\n--- Safe Operations ---");

    let result = a.checked_sub(&b);
    println!("a.checked_sub(&b) = {result:?}");

    #[cfg(not(feature = "non-zero"))]
    {
        // `sub_or_zero` floors at zero, and the name says so at the call site.
        // `saturating_sub` did the same thing but is deprecated: saturating
        // arithmetic hides underflow, which is data corruption in financial
        // code.
        let floored = b.sub_or_zero(&a.to_dec());
        println!("b.sub_or_zero(a) = {floored} (floors at ZERO)");
    }

    // Mathematical functions
    println!("\n--- Mathematical Functions ---");

    let value = pos_or_panic!(16.0);
    println!("value = {value}");
    println!("sqrt({value}) = {}", value.sqrt());
    println!("ln({value}) = {}", value.ln());
    println!("log10({value}) = {}", value.log10());
    println!("exp(1) = {}", Positive::ONE.exp());
    println!("pow({value}, 2) = {}", value.powi(2));

    // Rounding operations
    println!("\n--- Rounding Operations ---");

    let decimal_value = pos_or_panic!(123.456789);
    println!("value = {decimal_value}");
    println!("floor() = {}", decimal_value.floor());
    println!("ceiling() = {}", decimal_value.ceiling());
    println!("round() = {}", decimal_value.round());
    println!("round_to(2) = {}", decimal_value.round_to(2));

    // Conversions
    println!("\n--- Conversions ---");

    let p = pos_or_panic!(42.5);
    println!("Positive value: {p}");
    println!("to_f64(): {}", p.to_f64());
    println!("to_i64(): {:?}", i64::try_from(p));
    println!("to_dec(): {}", p.to_dec());

    // Comparisons
    println!("\n--- Comparisons ---");

    let x = pos_or_panic!(10.0);
    let y = pos_or_panic!(20.0);
    println!("x = {x}, y = {y}");
    println!("x < y: {}", x < y);
    println!("x.max(y) = {}", x.max(y));
    println!("x.min(y) = {}", x.min(y));

    // Clamping
    let value = pos_or_panic!(150.0);
    let clamped = value.clamp(pos_or_panic!(1.0), pos_or_panic!(100.0));
    println!("clamp({value}, 0, 100) = {clamped}");

    println!("\n=== Example Complete ===");
}
