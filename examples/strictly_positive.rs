/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 05/10/26
******************************************************************************/

//! `StrictlyPositive` (> 0) used side by side with `Positive` (>= 0).
//!
//! Run with `cargo run --example strictly_positive`. Like the other examples,
//! this one uses `strict_pos_or_panic!` / `pos_or_panic!` for brevity; production
//! code should use the fallible constructors and handle the error.

use positive::prelude::*;
use rust_decimal_macros::dec;

/// The motivating example from issue #122: a price can never be zero, a daily
/// volume can.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Instrument {
    price: StrictlyPositive, // > 0
    daily_volume: Positive,  // >= 0
}

fn main() {
    println!("=== StrictlyPositive ===\n");

    let instrument = Instrument {
        price: strict_pos_or_panic!(101.25),
        daily_volume: pos_or_panic!(1500.0),
    };
    println!("Instrument: {instrument:?}");
    let json = serde_json::to_string(&instrument).unwrap();
    println!("JSON:       {json}");

    // Zero is rejected.
    println!("\n--- Construction ---");
    println!("strict_pos!(0.0)  = {:?}", strict_pos!(0.0));
    println!("strict_pos!(-1.0) = {:?}", strict_pos!(-1.0));
    println!(
        "from \"0.5\"        = {:?}",
        "0.5".parse::<StrictlyPositive>()
    );
    let rejected = serde_json::from_str::<StrictlyPositive>("\"0\"");
    println!("deserialise \"0\"   = {rejected:?}");

    // Arithmetic keeps the strict type where the result is guaranteed > 0.
    println!("\n--- Arithmetic ---");
    let price = instrument.price;
    let fee = pos_or_panic!(0.75);
    let with_fee: StrictlyPositive = price + fee; // S + P -> S
    let notional: Positive = price * instrument.daily_volume; // S * P -> P
    let spread: Positive = with_fee - price; // S - S -> P (may be zero)
    println!("price + fee        = {with_fee}");
    println!("price * volume     = {notional}");
    println!("(price + fee) - p  = {spread}");

    // `checked_sub` keeps the strict type and reports a non-positive result.
    println!("price.checked_sub(price) = {:?}", price.checked_sub(&price));

    // Products and quotients of tiny values can underflow to zero in Decimal.
    let tiny = StrictlyPositive::new_decimal(dec!(1e-20)).unwrap();
    println!("tiny.checked_mul(tiny)   = {:?}", tiny.checked_mul(&tiny));

    // Logarithms have no zero edge case.
    println!("\n--- Maths ---");
    println!("ln(price)    = {}", price.ln());
    println!("sqrt(price)  = {}", price.sqrt().round_to(6));

    // Converting between the two types.
    println!("\n--- Conversions ---");
    let as_positive: Positive = price.into();
    println!("into Positive         = {as_positive}");
    println!(
        "try_from(Positive::ONE) = {:?}",
        StrictlyPositive::try_from(Positive::ONE)
    );
}
