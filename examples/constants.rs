/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Example demonstrating the use of predefined constants.
//!
//! This example shows all available constants in the Positive library.

use positive::prelude::*;

fn main() {
    println!("=== Positive Constants ===\n");

    // Integer constants (0-10)
    println!("--- Integer Constants (0-10) ---");
    #[cfg(not(feature = "non-zero"))]
    println!("ZERO     = {}", ZERO);
    println!("ONE      = {}", ONE);
    println!("TWO      = {}", TWO);
    println!("THREE    = {}", THREE);
    println!("FOUR     = {}", FOUR);
    println!("FIVE     = {}", FIVE);
    println!("SIX      = {}", SIX);
    println!("SEVEN    = {}", SEVEN);
    println!("EIGHT    = {}", EIGHT);
    println!("NINE     = {}", NINE);
    println!("TEN      = {}", TEN);

    // Multiples of 5 (15-95)
    println!("\n--- Multiples of 5 (15-95) ---");
    println!("FIFTEEN      = {}", FIFTEEN);
    println!("TWENTY       = {}", TWENTY);
    println!("TWENTY_FIVE  = {}", TWENTY_FIVE);
    println!("THIRTY       = {}", THIRTY);
    println!("FORTY        = {}", FORTY);
    println!("FIFTY        = {}", FIFTY);
    println!("SIXTY        = {}", SIXTY);
    println!("SEVENTY      = {}", SEVENTY);
    println!("EIGHTY       = {}", EIGHTY);
    println!("NINETY       = {}", NINETY);
    println!("NINETY_FIVE  = {}", NINETY_FIVE);

    // Multiples of 100 (100-900)
    println!("\n--- Multiples of 100 (100-900) ---");
    println!("HUNDRED       = {}", HUNDRED);
    println!("TWO_HUNDRED   = {}", TWO_HUNDRED);
    println!("THREE_HUNDRED = {}", THREE_HUNDRED);
    println!("FIVE_HUNDRED  = {}", FIVE_HUNDRED);

    // Multiples of 1000 (1000-10000)
    println!("\n--- Multiples of 1000 (1000-10000) ---");
    println!("THOUSAND      = {}", THOUSAND);
    println!("TWO_THOUSAND  = {}", TWO_THOUSAND);
    println!("FIVE_THOUSAND = {}", FIVE_THOUSAND);
    println!("TEN_THOUSAND  = {}", TEN_THOUSAND);

    // Mathematical constants
    println!("\n--- Mathematical Constants ---");
    println!("PI = {}", PI);
    println!("E  = {}", E);

    // Special values
    println!("\n--- Special Values ---");
    println!("MAX = {}", MAX);
    println!("EPSILON  = {}", EPSILON);

    // Using constants in calculations
    println!("\n--- Using Constants in Calculations ---");

    let radius = FIVE;
    let area = PI * radius.powi(2);
    println!("Circle area (r=5): PI * 5² = {:.4}", area);

    let percentage = FIFTY / HUNDRED;
    println!("50% as decimal: {}", percentage);

    let compound = (ONE + pos_or_panic!(0.05)).powi(i64::try_from(TEN).expect("TEN fits in i64"));
    println!("Compound interest (5% for 10 years): {:.4}", compound);

    // Constants are also available via Positive::CONSTANT
    println!("\n--- Alternative Access via Positive::CONSTANT ---");
    #[cfg(not(feature = "non-zero"))]
    println!("Positive::ZERO = {}", Positive::ZERO);
    println!("Positive::PI   = {}", Positive::PI);
    println!("Positive::E    = {}", Positive::E);

    println!("\n=== Example Complete ===");
}
