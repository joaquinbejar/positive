/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Example demonstrating error handling with Positive values.
//!
//! This example shows how to handle errors when creating and manipulating
//! Positive values.

use positive::prelude::*;

fn main() {
    println!("=== Error Handling with Positive ===\n");

    // Handling creation errors
    println!("--- Creation Errors ---");

    // Using pos! macro (returns Result)
    match pos!(5.0) {
        Ok(value) => println!("Created positive value: {value}"),
        Err(e) => println!("Error: {e}"),
    }

    match pos!(-5.0) {
        Ok(value) => println!("Created positive value: {value}"),
        Err(e) => println!("Error creating negative: {e}"),
    }

    // Using spos! for optional handling
    println!("\n--- Optional Handling with spos! ---");

    let valid = spos!(10.0);
    let invalid = spos!(-10.0);

    println!("spos!(10.0)  = {valid:?}");
    println!("spos!(-10.0) = {invalid:?}");

    // Using if-let pattern
    if let Some(value) = spos!(42.0) {
        println!("Got value: {value}");
    }

    // Checked operations
    println!("\n--- Checked Operations ---");

    let a = pos_or_panic!(10.0);
    let b = pos_or_panic!(3.0);

    // checked_sub returns Result
    match a.checked_sub(&b) {
        Ok(result) => println!("{a} - {b} = {result}"),
        Err(e) => println!("Subtraction error: {e}"),
    }

    // This would fail: subtracting larger from smaller
    match b.checked_sub(&a) {
        Ok(result) => println!("{b} - {a} = {result}"),
        Err(e) => println!("Subtraction error: {e}"),
    }

    // checked_div handles division by zero
    println!("\n--- Division Safety ---");

    match a.checked_div(&b) {
        Ok(result) => println!("{a} / {b} = {result}"),
        Err(e) => println!("Division error: {e}"),
    }

    #[cfg(not(feature = "non-zero"))]
    match a.checked_div(&ZERO) {
        Ok(result) => println!("{a} / 0 = {result}"),
        Err(e) => println!("Division by zero error: {e}"),
    }

    // Flooring at zero, explicitly (never fails)
    println!("\n--- Flooring At Zero ---");

    #[cfg(not(feature = "non-zero"))]
    #[allow(deprecated)]
    {
        let small = pos_or_panic!(5.0);
        let large = pos_or_panic!(100.0);
        let result = small.sub_or_zero(&large.to_dec());
        println!("{small}.sub_or_zero({large}) = {result} (floors at ZERO)");
    }

    // Using Result combinators
    println!("\n--- Using Result Combinators ---");

    let result: PositiveResult<Positive> = pos!(25.0).and_then(|v| {
        let doubled = v * TWO;
        Ok(doubled)
    });
    println!("pos!(25.0) doubled: {result:?}");

    // Chaining operations with ?
    println!("\n--- Chaining with ? operator ---");

    fn calculate_discount(price: f64, discount_percent: f64) -> PositiveResult<Positive> {
        let price = Positive::new(price)?;
        let discount = Positive::new(discount_percent)?;
        let discount_amount = price * discount / HUNDRED;
        price.checked_sub(&discount_amount)
    }

    match calculate_discount(100.0, 20.0) {
        Ok(final_price) => println!("Price after 20% discount: {final_price}"),
        Err(e) => println!("Calculation error: {e}"),
    }

    match calculate_discount(100.0, 150.0) {
        Ok(final_price) => println!("Price after 150% discount: {final_price}"),
        Err(e) => println!("Calculation error (150% discount): {e}"),
    }

    // Type checking
    println!("\n--- Type Checking ---");

    println!("is_positive::<Positive>() = {}", is_positive::<Positive>());
    println!("is_positive::<f64>()      = {}", is_positive::<f64>());
    println!("is_positive::<i32>()      = {}", is_positive::<i32>());

    println!("\n=== Example Complete ===");
}
