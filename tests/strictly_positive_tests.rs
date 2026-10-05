/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 05/10/26
******************************************************************************/

//! Integration tests for `StrictlyPositive`.
//!
//! The strict invariant (`> 0`) does not depend on the `non-zero` feature, so
//! almost every test here runs unchanged in all three configurations. The few
//! that need a zero `Positive` are gated on `not(feature = "non-zero")`, where
//! such a value exists.

use approx::{abs_diff_eq, relative_eq};
use positive::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

/// Builds a `StrictlyPositive` from a literal the test knows to be valid.
fn sp(value: Decimal) -> StrictlyPositive {
    match StrictlyPositive::new_decimal(value) {
        Ok(v) => v,
        Err(e) => panic!("test literal {value} must be strictly positive: {e}"),
    }
}

/// Builds a `Positive` from a literal the test knows to be valid.
fn p(value: Decimal) -> Positive {
    match Positive::new_decimal(value) {
        Ok(v) => v,
        Err(e) => panic!("test literal {value} must be positive: {e}"),
    }
}

/// The error every zero/negative rejection must carry, in every feature mode.
fn strict_oob(value: Decimal) -> PositiveError {
    PositiveError::OutOfBounds {
        value,
        min: dec!(1e-28),
        max: Decimal::MAX,
    }
}

// ---------------------------------------------------------------------------
// The motivating example from #122
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Instrument {
    price: StrictlyPositive, // > 0
    daily_volume: Positive,  // >= 0
}

#[test]
fn test_instrument_mixed_fields_compiles_and_round_trips() {
    let instrument = Instrument {
        price: sp(dec!(101.25)),
        daily_volume: p(dec!(1500)),
    };
    let json = serde_json::to_string(&instrument);
    assert_eq!(
        json.as_deref().ok(),
        Some(r#"{"price":"101.25","daily_volume":"1500"}"#)
    );
    let back: Result<Instrument, _> =
        serde_json::from_str(r#"{"price":"101.25","daily_volume":"1500"}"#);
    assert!(matches!(back, Ok(ref i) if *i == instrument));
}

#[test]
fn test_instrument_zero_price_rejected_on_deserialize() {
    let back: Result<Instrument, _> =
        serde_json::from_str(r#"{"price":"0","daily_volume":"1500"}"#);
    assert!(back.is_err());
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_instrument_zero_volume_accepted_on_deserialize() {
    let back: Result<Instrument, _> = serde_json::from_str(r#"{"price":"1","daily_volume":"0"}"#);
    assert!(matches!(back, Ok(ref i) if i.daily_volume.is_zero()));
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

#[test]
fn test_new_positive_returns_ok() {
    assert!(matches!(StrictlyPositive::new(1.5), Ok(v) if v.to_dec() == dec!(1.5)));
}

#[test]
fn test_new_zero_returns_out_of_bounds() {
    assert_eq!(StrictlyPositive::new(0.0), Err(strict_oob(dec!(0))));
}

#[test]
fn test_new_negative_zero_returns_out_of_bounds() {
    assert!(matches!(
        StrictlyPositive::new(-0.0),
        Err(PositiveError::OutOfBounds { .. })
    ));
}

#[test]
fn test_new_negative_returns_out_of_bounds() {
    assert_eq!(StrictlyPositive::new(-1.0), Err(strict_oob(dec!(-1))));
}

#[test]
fn test_new_nan_returns_invalid_value() {
    assert!(matches!(
        StrictlyPositive::new(f64::NAN),
        Err(PositiveError::InvalidValue { .. })
    ));
}

#[test]
fn test_new_infinity_returns_invalid_value() {
    assert!(matches!(
        StrictlyPositive::new(f64::INFINITY),
        Err(PositiveError::InvalidValue { .. })
    ));
    assert!(matches!(
        StrictlyPositive::new(f64::NEG_INFINITY),
        Err(PositiveError::InvalidValue { .. })
    ));
}

#[test]
fn test_new_out_of_decimal_range_returns_conversion_error() {
    assert!(matches!(
        StrictlyPositive::new(1e300),
        Err(PositiveError::ConversionError { .. })
    ));
}

#[test]
fn test_new_decimal_positive_returns_ok() {
    assert!(matches!(StrictlyPositive::new_decimal(dec!(0.01)), Ok(v) if v == dec!(0.01)));
}

#[test]
fn test_new_decimal_min_returns_ok() {
    assert_eq!(
        StrictlyPositive::new_decimal(dec!(1e-28)),
        Ok(StrictlyPositive::MIN)
    );
}

#[test]
fn test_new_decimal_max_returns_ok() {
    assert_eq!(
        StrictlyPositive::new_decimal(Decimal::MAX),
        Ok(StrictlyPositive::MAX)
    );
}

#[test]
fn test_new_decimal_zero_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::new_decimal(Decimal::ZERO),
        Err(strict_oob(dec!(0)))
    );
}

#[test]
fn test_new_decimal_negative_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::new_decimal(dec!(-0.5)),
        Err(strict_oob(dec!(-0.5)))
    );
}

#[test]
fn test_out_of_bounds_message_reports_strict_minimum() {
    let message = match StrictlyPositive::new_decimal(Decimal::ZERO) {
        Err(e) => e.to_string(),
        Ok(v) => panic!("zero accepted as {v}"),
    };
    assert_eq!(
        message,
        format!(
            "value 0 is out of bounds (min: 0.0000000000000000000000000001, max: {})",
            Decimal::MAX
        )
    );
}

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

#[test]
fn test_strict_pos_macro_valid_returns_ok() {
    assert!(matches!(strict_pos!(2.5), Ok(v) if v == 2.5));
}

#[test]
fn test_strict_pos_macro_zero_returns_err() {
    assert!(matches!(
        strict_pos!(0.0),
        Err(PositiveError::OutOfBounds { .. })
    ));
}

#[test]
fn test_strict_pos_or_panic_macro_valid_returns_value() {
    assert_eq!(strict_pos_or_panic!(3.0), StrictlyPositive::THREE);
}

#[test]
#[should_panic(expected = "Failed to create StrictlyPositive value")]
fn test_strict_pos_or_panic_macro_zero_panics() {
    let _ = strict_pos_or_panic!(0.0);
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn test_constants_values_match_positive() {
    let pairs = [
        (StrictlyPositive::ONE, Positive::ONE),
        (StrictlyPositive::TWO, Positive::TWO),
        (StrictlyPositive::THREE, Positive::THREE),
        (StrictlyPositive::FOUR, Positive::FOUR),
        (StrictlyPositive::FIVE, Positive::FIVE),
        (StrictlyPositive::SIX, Positive::SIX),
        (StrictlyPositive::SEVEN, Positive::SEVEN),
        (StrictlyPositive::EIGHT, Positive::EIGHT),
        (StrictlyPositive::NINE, Positive::NINE),
        (StrictlyPositive::TEN, Positive::TEN),
        (StrictlyPositive::HUNDRED, Positive::HUNDRED),
        (StrictlyPositive::THOUSAND, Positive::THOUSAND),
        (StrictlyPositive::PI, Positive::PI),
        (StrictlyPositive::E, Positive::E),
        (StrictlyPositive::MAX, Positive::MAX),
    ];
    for (strict, positive) in pairs {
        assert_eq!(strict, positive);
        assert_eq!(strict.to_positive(), positive);
    }
}

#[test]
fn test_constants_min_is_one_e_minus_28() {
    assert_eq!(StrictlyPositive::MIN.to_dec(), dec!(1e-28));
    assert_eq!(StrictlyPositive::MIN.to_dec(), Decimal::new(1, 28));
}

// ---------------------------------------------------------------------------
// Accessors and conversions
// ---------------------------------------------------------------------------

#[test]
fn test_accessors_return_inner_value() {
    let v = sp(dec!(2.5));
    assert_eq!(v.value(), dec!(2.5));
    assert_eq!(v.to_dec(), dec!(2.5));
    assert_eq!(v.to_f64(), 2.5);
    assert_eq!(v.to_positive(), p(dec!(2.5)));
    let as_ref: &Positive = v.as_ref();
    assert_eq!(*as_ref, p(dec!(2.5)));
}

#[test]
fn test_integer_checked_accessors_truncate_and_detect_overflow() {
    let v = sp(dec!(42.9));
    assert_eq!(v.to_i64_checked(), Some(42));
    assert_eq!(v.to_u64_checked(), Some(42));
    assert_eq!(v.to_usize_checked(), Some(42));
    assert_eq!(StrictlyPositive::MAX.to_i64_checked(), None);
    assert_eq!(StrictlyPositive::MAX.to_u64_checked(), None);
    assert_eq!(StrictlyPositive::MAX.to_usize_checked(), None);
}

#[test]
fn test_from_strictly_positive_into_positive_decimal_f64() {
    let v = sp(dec!(7.25));
    let positive: Positive = v.into();
    let decimal: Decimal = v.into();
    let float: f64 = v.into();
    assert_eq!(positive, p(dec!(7.25)));
    assert_eq!(decimal, dec!(7.25));
    assert_eq!(float, 7.25);
}

#[test]
fn test_try_from_positive_nonzero_returns_ok() {
    assert_eq!(
        StrictlyPositive::try_from(Positive::FIVE),
        Ok(StrictlyPositive::FIVE)
    );
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_try_from_positive_zero_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::try_from(Positive::ZERO),
        Err(strict_oob(dec!(0)))
    );
}

#[test]
fn test_try_from_decimal_and_ref() {
    assert_eq!(
        StrictlyPositive::try_from(dec!(3)),
        Ok(StrictlyPositive::THREE)
    );
    assert_eq!(
        StrictlyPositive::try_from(&dec!(3)),
        Ok(StrictlyPositive::THREE)
    );
    assert_eq!(
        StrictlyPositive::try_from(Decimal::ZERO),
        Err(strict_oob(dec!(0)))
    );
    assert_eq!(
        StrictlyPositive::try_from(&dec!(-3)),
        Err(strict_oob(dec!(-3)))
    );
}

#[test]
fn test_try_from_f64() {
    assert_eq!(StrictlyPositive::try_from(4.0), Ok(StrictlyPositive::FOUR));
    assert_eq!(StrictlyPositive::try_from(0.0), Err(strict_oob(dec!(0))));
    assert!(matches!(
        StrictlyPositive::try_from(f64::NAN),
        Err(PositiveError::InvalidValue { .. })
    ));
}

#[test]
fn test_try_from_i64() {
    assert_eq!(StrictlyPositive::try_from(5i64), Ok(StrictlyPositive::FIVE));
    assert_eq!(StrictlyPositive::try_from(0i64), Err(strict_oob(dec!(0))));
    assert_eq!(StrictlyPositive::try_from(-5i64), Err(strict_oob(dec!(-5))));
}

#[test]
fn test_try_from_u64() {
    assert_eq!(StrictlyPositive::try_from(5u64), Ok(StrictlyPositive::FIVE));
    assert_eq!(StrictlyPositive::try_from(0u64), Err(strict_oob(dec!(0))));
    let big = u64::MAX;
    assert!(matches!(StrictlyPositive::try_from(big), Ok(v) if v.to_dec() == Decimal::from(big)));
}

#[test]
fn test_try_from_usize_is_exact() {
    assert_eq!(StrictlyPositive::try_from(0usize), Err(strict_oob(dec!(0))));
    // `usize::MAX` is portable across 32/64-bit targets; on 64-bit it is
    // also an integer that `f64` cannot represent exactly.
    let value = usize::MAX;
    assert!(
        matches!(StrictlyPositive::try_from(value), Ok(v) if v.to_dec() == Decimal::from(value))
    );
}

#[test]
fn test_try_into_integers() {
    let v = sp(dec!(42.9));
    assert_eq!(u64::try_from(v), Ok(42));
    assert_eq!(i64::try_from(v), Ok(42));
    assert_eq!(usize::try_from(v), Ok(42));
    assert!(matches!(
        u64::try_from(StrictlyPositive::MAX),
        Err(PositiveError::ConversionError { .. })
    ));
    assert!(matches!(
        i64::try_from(StrictlyPositive::MAX),
        Err(PositiveError::ConversionError { .. })
    ));
    assert!(matches!(
        usize::try_from(StrictlyPositive::MAX),
        Err(PositiveError::ConversionError { .. })
    ));
}

#[test]
fn test_from_str_valid_returns_ok() {
    assert_eq!(StrictlyPositive::from_str("1.5"), Ok(sp(dec!(1.5))));
    assert_eq!(
        "0.0000000000000000000000000001".parse(),
        Ok(StrictlyPositive::MIN)
    );
}

#[test]
fn test_from_str_zero_and_negative_return_out_of_bounds() {
    assert_eq!(StrictlyPositive::from_str("0"), Err(strict_oob(dec!(0))));
    assert_eq!(
        StrictlyPositive::from_str("0.000"),
        Err(strict_oob(dec!(0)))
    );
    assert_eq!(StrictlyPositive::from_str("-2"), Err(strict_oob(dec!(-2))));
}

#[test]
fn test_from_str_garbage_returns_invalid_value() {
    assert!(matches!(
        StrictlyPositive::from_str("abc"),
        Err(PositiveError::InvalidValue { .. })
    ));
}

#[test]
fn test_display_and_debug_match_positive() {
    let v = sp(dec!(4.578923789423789));
    let positive = v.to_positive();
    assert_eq!(format!("{v}"), format!("{positive}"));
    assert_eq!(format!("{v:.2}"), "4.57");
    assert_eq!(format!("{v:?}"), format!("{positive:?}"));
    assert_eq!(format!("{:?}", sp(dec!(5.000))), "5");
    assert_eq!(format!("{}", sp(dec!(1.500))), "1.5");
}

// ---------------------------------------------------------------------------
// Comparisons
// ---------------------------------------------------------------------------

#[test]
fn test_eq_and_ord_with_positive_both_directions() {
    let s = StrictlyPositive::TWO;
    assert!(s == Positive::TWO);
    assert!(Positive::TWO == s);
    assert!(s < Positive::THREE);
    assert!(Positive::THREE > s);
    assert!(s > Positive::ONE);
    assert!(Positive::ONE < s);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_strictly_positive_greater_than_positive_zero() {
    assert!(StrictlyPositive::MIN > Positive::ZERO);
    assert!(Positive::ZERO < StrictlyPositive::MIN);
}

#[test]
fn test_eq_and_ord_with_decimal_both_directions() {
    let s = sp(dec!(2.5));
    assert!(s == dec!(2.5));
    assert!(dec!(2.5) == s);
    assert!(s > dec!(0));
    assert!(dec!(0) < s);
    assert!(s < dec!(3));
    assert!(dec!(3) > s);
}

#[test]
fn test_eq_and_ord_with_f64_both_directions() {
    let s = sp(dec!(2.5));
    assert!(s == 2.5);
    assert!(2.5 == s);
    assert!(s > 0.0);
    assert!(0.0 < s);
    assert!(s < f64::INFINITY);
    assert!(s.partial_cmp(&f64::NAN).is_none());
    assert!(f64::NAN.partial_cmp(&s).is_none());
}

#[test]
fn test_ord_and_hash_are_consistent() {
    let a = sp(dec!(1.50));
    let b = sp(dec!(1.5));
    assert_eq!(a, b);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
    let mut values = vec![
        StrictlyPositive::THREE,
        StrictlyPositive::ONE,
        StrictlyPositive::TWO,
    ];
    values.sort();
    assert_eq!(
        values,
        vec![
            StrictlyPositive::ONE,
            StrictlyPositive::TWO,
            StrictlyPositive::THREE
        ]
    );
}

#[test]
fn test_approx_delegates_to_positive() {
    let a = sp(dec!(1.0));
    let b = sp(dec!(1.00000000000000001));
    assert!(abs_diff_eq!(a, b));
    assert!(relative_eq!(a, b));
    assert!(!abs_diff_eq!(a, StrictlyPositive::TWO));
    assert!(abs_diff_eq!(a, StrictlyPositive::TWO, epsilon = dec!(1)));
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------

#[test]
fn test_serde_round_trip_exact_string() {
    let v = sp(dec!(0.1234567890123456789012345678));
    let json = serde_json::to_string(&v);
    assert_eq!(
        json.as_deref().ok(),
        Some("\"0.1234567890123456789012345678\"")
    );
    let back: Result<StrictlyPositive, _> =
        serde_json::from_str("\"0.1234567890123456789012345678\"");
    assert!(matches!(back, Ok(b) if b == v));
}

#[test]
fn test_serde_round_trip_max() {
    let json = serde_json::to_string(&StrictlyPositive::MAX);
    let back: Result<StrictlyPositive, _> = match json {
        Ok(ref s) => serde_json::from_str(s),
        Err(e) => panic!("serialisation failed: {e}"),
    };
    assert!(matches!(back, Ok(b) if b == StrictlyPositive::MAX));
}

#[test]
fn test_serde_wire_format_matches_positive() {
    let s = serde_json::to_string(&sp(dec!(42.5)));
    let p = serde_json::to_string(&p(dec!(42.5)));
    assert!(s.is_ok());
    assert_eq!(s.ok(), p.ok());
}

#[test]
fn test_serde_zero_string_rejected() {
    let back: Result<StrictlyPositive, _> = serde_json::from_str("\"0\"");
    assert!(back.is_err());
    let back: Result<StrictlyPositive, _> = serde_json::from_str("\"0.00\"");
    assert!(back.is_err());
}

#[test]
fn test_serde_zero_number_rejected() {
    let back: Result<StrictlyPositive, _> = serde_json::from_str("0");
    assert!(back.is_err());
    let back: Result<StrictlyPositive, _> = serde_json::from_str("0.0");
    assert!(back.is_err());
}

#[test]
fn test_serde_negative_rejected() {
    let back: Result<StrictlyPositive, _> = serde_json::from_str("\"-1\"");
    assert!(back.is_err());
    let back: Result<StrictlyPositive, _> = serde_json::from_str("-1");
    assert!(back.is_err());
}

#[test]
fn test_serde_legacy_number_accepted() {
    let back: Result<StrictlyPositive, _> = serde_json::from_str("1.5");
    assert!(matches!(back, Ok(v) if v == dec!(1.5)));
    let back: Result<StrictlyPositive, _> = serde_json::from_str("42");
    assert!(matches!(back, Ok(v) if v == dec!(42)));
}

#[test]
fn test_serde_garbage_rejected() {
    let back: Result<StrictlyPositive, _> = serde_json::from_str("\"abc\"");
    assert!(back.is_err());
}

// ---------------------------------------------------------------------------
// Addition
// ---------------------------------------------------------------------------

#[test]
fn test_add_strict_strict_returns_strict() {
    let sum: StrictlyPositive = StrictlyPositive::ONE + StrictlyPositive::TWO;
    assert_eq!(sum, StrictlyPositive::THREE);
    assert_eq!(
        StrictlyPositive::ONE.checked_add(&StrictlyPositive::TWO),
        Ok(StrictlyPositive::THREE)
    );
}

#[test]
fn test_checked_add_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_add(&StrictlyPositive::ONE),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
#[should_panic(expected = "overflow")]
fn test_add_overflow_panics() {
    let _ = StrictlyPositive::MAX + StrictlyPositive::ONE;
}

#[test]
fn test_add_strict_positive_both_directions_returns_strict() {
    let a: StrictlyPositive = StrictlyPositive::ONE + Positive::TWO;
    let b: StrictlyPositive = Positive::TWO + StrictlyPositive::ONE;
    assert_eq!(a, StrictlyPositive::THREE);
    assert_eq!(b, StrictlyPositive::THREE);
    assert_eq!(
        StrictlyPositive::ONE.checked_add_positive(&Positive::TWO),
        Ok(StrictlyPositive::THREE)
    );
}

#[test]
fn test_checked_add_positive_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_add_positive(&Positive::ONE),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_add_positive_zero_keeps_value() {
    assert_eq!(
        StrictlyPositive::MIN + Positive::ZERO,
        StrictlyPositive::MIN
    );
    assert_eq!(
        Positive::ZERO + StrictlyPositive::MIN,
        StrictlyPositive::MIN
    );
}

#[test]
#[should_panic(expected = "overflow")]
fn test_add_positive_overflow_panics() {
    let _ = Positive::MAX + StrictlyPositive::ONE;
}

#[test]
fn test_add_assign_strict_and_positive() {
    let mut v = StrictlyPositive::ONE;
    v += StrictlyPositive::TWO;
    assert_eq!(v, StrictlyPositive::THREE);
    v += Positive::TWO;
    assert_eq!(v, StrictlyPositive::FIVE);
}

#[test]
#[should_panic(expected = "overflow")]
fn test_add_assign_overflow_panics() {
    let mut v = StrictlyPositive::MAX;
    v += StrictlyPositive::ONE;
}

// ---------------------------------------------------------------------------
// Subtraction
// ---------------------------------------------------------------------------

#[test]
fn test_sub_strict_strict_returns_positive() {
    let diff: Positive = StrictlyPositive::THREE - StrictlyPositive::ONE;
    assert_eq!(diff, Positive::TWO);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sub_equal_values_returns_positive_zero() {
    let diff: Positive = StrictlyPositive::TWO - StrictlyPositive::TWO;
    assert!(diff.is_zero());
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "invariant broken in sub")]
fn test_sub_equal_values_panics_under_non_zero() {
    let _ = StrictlyPositive::TWO - StrictlyPositive::TWO;
}

#[test]
#[should_panic(expected = "invariant broken in sub")]
fn test_sub_negative_result_panics() {
    let _ = StrictlyPositive::ONE - StrictlyPositive::TWO;
}

#[test]
fn test_checked_sub_positive_result_returns_ok() {
    assert_eq!(
        StrictlyPositive::THREE.checked_sub(&StrictlyPositive::ONE),
        Ok(StrictlyPositive::TWO)
    );
    assert_eq!(
        sp(dec!(1e-27)).checked_sub(&StrictlyPositive::MIN),
        Ok(sp(dec!(9e-28)))
    );
}

#[test]
fn test_checked_sub_zero_result_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::TWO.checked_sub(&StrictlyPositive::TWO),
        Err(strict_oob(dec!(0)))
    );
}

#[test]
fn test_checked_sub_negative_result_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::ONE.checked_sub(&StrictlyPositive::THREE),
        Err(strict_oob(dec!(-2)))
    );
}

// ---------------------------------------------------------------------------
// Multiplication
// ---------------------------------------------------------------------------

#[test]
fn test_mul_strict_strict_returns_strict() {
    let product: StrictlyPositive = StrictlyPositive::TWO * StrictlyPositive::THREE;
    assert_eq!(product, StrictlyPositive::SIX);
    assert_eq!(
        StrictlyPositive::TWO.checked_mul(&StrictlyPositive::THREE),
        Ok(StrictlyPositive::SIX)
    );
}

#[test]
fn test_checked_mul_underflow_to_zero_returns_out_of_bounds() {
    let tiny = sp(dec!(1e-20));
    assert_eq!(tiny.checked_mul(&tiny), Err(strict_oob(dec!(0))));
    assert!(matches!(
        StrictlyPositive::MIN.checked_mul(&sp(dec!(0.1))),
        Err(PositiveError::OutOfBounds { .. })
    ));
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in mul")]
fn test_mul_underflow_to_zero_panics() {
    let tiny = sp(dec!(1e-20));
    let _ = tiny * tiny;
}

#[test]
fn test_checked_mul_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_mul(&StrictlyPositive::TWO),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
#[should_panic(expected = "overflow")]
fn test_mul_overflow_panics() {
    let _ = StrictlyPositive::MAX * StrictlyPositive::TWO;
}

#[test]
fn test_mul_strict_positive_both_directions_returns_positive() {
    let a: Positive = StrictlyPositive::TWO * Positive::THREE;
    let b: Positive = Positive::THREE * StrictlyPositive::TWO;
    assert_eq!(a, Positive::SIX);
    assert_eq!(b, Positive::SIX);
    assert_eq!(
        StrictlyPositive::TWO.checked_mul_positive(&Positive::THREE),
        Ok(Positive::SIX)
    );
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_mul_by_positive_zero_returns_zero() {
    let a: Positive = StrictlyPositive::TWO * Positive::ZERO;
    let b: Positive = Positive::ZERO * StrictlyPositive::TWO;
    assert!(a.is_zero());
    assert!(b.is_zero());
}

#[test]
fn test_checked_mul_positive_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_mul_positive(&Positive::TWO),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
fn test_mul_assign_strict() {
    let mut v = StrictlyPositive::TWO;
    v *= StrictlyPositive::FIVE;
    assert_eq!(v, StrictlyPositive::TEN);
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in mul_assign")]
fn test_mul_assign_underflow_panics() {
    let mut v = StrictlyPositive::MIN;
    v *= StrictlyPositive::MIN;
}

// ---------------------------------------------------------------------------
// Division
// ---------------------------------------------------------------------------

#[test]
fn test_div_strict_strict_returns_strict() {
    let q: StrictlyPositive = StrictlyPositive::SIX / StrictlyPositive::TWO;
    assert_eq!(q, StrictlyPositive::THREE);
    assert_eq!(
        StrictlyPositive::SIX.checked_div(&StrictlyPositive::TWO),
        Ok(StrictlyPositive::THREE)
    );
}

#[test]
fn test_div_rounding_matches_positive() {
    let strict = StrictlyPositive::ONE / StrictlyPositive::THREE;
    let positive = Positive::ONE / Positive::THREE;
    assert_eq!(strict, positive);
    assert_eq!(strict.to_dec(), dec!(0.3333333333333333333333333333));
}

#[test]
fn test_checked_div_underflow_to_zero_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::MIN.checked_div(&sp(dec!(10000000000))),
        Err(strict_oob(dec!(0)))
    );
    assert!(matches!(
        StrictlyPositive::MIN.checked_div(&StrictlyPositive::THREE),
        Err(PositiveError::OutOfBounds { .. })
    ));
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in div")]
fn test_div_underflow_to_zero_panics() {
    let _ = StrictlyPositive::MIN / StrictlyPositive::TEN;
}

#[test]
fn test_checked_div_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_div(&StrictlyPositive::MIN),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
#[should_panic(expected = "overflow")]
fn test_div_overflow_panics() {
    let _ = StrictlyPositive::MAX / StrictlyPositive::MIN;
}

#[test]
fn test_div_strict_by_positive_returns_positive() {
    let q: Positive = StrictlyPositive::SIX / Positive::TWO;
    assert_eq!(q, Positive::THREE);
    assert_eq!(
        StrictlyPositive::SIX.checked_div_positive(&Positive::TWO),
        Ok(Positive::THREE)
    );
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_checked_div_positive_zero_divisor_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::SIX.checked_div_positive(&Positive::ZERO),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[cfg(not(feature = "non-zero"))]
#[test]
#[should_panic(expected = "invariant broken in div")]
fn test_div_strict_by_positive_zero_panics() {
    let _ = StrictlyPositive::SIX / Positive::ZERO;
}

#[test]
fn test_div_positive_by_strict_returns_positive() {
    let q: Positive = Positive::SIX / StrictlyPositive::TWO;
    assert_eq!(q, Positive::THREE);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_div_positive_zero_by_strict_returns_zero() {
    let q: Positive = Positive::ZERO / StrictlyPositive::TWO;
    assert!(q.is_zero());
}

// ---------------------------------------------------------------------------
// Sum
// ---------------------------------------------------------------------------

#[test]
fn test_checked_sum_owned_and_borrowed() {
    let values = [
        StrictlyPositive::ONE,
        StrictlyPositive::TWO,
        StrictlyPositive::THREE,
    ];
    assert_eq!(
        StrictlyPositive::checked_sum(values),
        Ok(StrictlyPositive::SIX)
    );
    assert_eq!(
        StrictlyPositive::checked_sum(values.iter()),
        Ok(StrictlyPositive::SIX)
    );
    assert_eq!(
        StrictlyPositive::checked_sum([StrictlyPositive::MIN]),
        Ok(StrictlyPositive::MIN)
    );
}

#[test]
fn test_checked_sum_empty_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::checked_sum(Vec::<StrictlyPositive>::new()),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
fn test_checked_sum_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::checked_sum([StrictlyPositive::MAX, StrictlyPositive::ONE]),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

// ---------------------------------------------------------------------------
// Mathematical functions
// ---------------------------------------------------------------------------

#[test]
fn test_sqrt_returns_strict() {
    assert_eq!(sp(dec!(16)).sqrt(), StrictlyPositive::FOUR);
    assert_eq!(
        StrictlyPositive::FOUR.checked_sqrt(),
        Ok(StrictlyPositive::TWO)
    );
    assert!(matches!(StrictlyPositive::MIN.checked_sqrt(), Ok(v) if v > dec!(0)));
}

#[test]
fn test_ln_and_log10_have_no_zero_domain_error() {
    assert_eq!(StrictlyPositive::ONE.ln(), Decimal::ZERO);
    assert!(sp(dec!(0.5)).ln() < Decimal::ZERO);
    assert_eq!(StrictlyPositive::HUNDRED.log10(), dec!(2));
    assert!(StrictlyPositive::MIN.ln() < Decimal::ZERO);
    assert!(StrictlyPositive::MIN.log10() < Decimal::ZERO);
}

#[test]
fn test_exp_returns_strict() {
    assert!(StrictlyPositive::ONE.exp() > StrictlyPositive::TWO);
    assert!(StrictlyPositive::ONE.checked_exp().is_ok());
}

#[test]
fn test_checked_exp_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::THOUSAND.checked_exp(),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
#[should_panic(expected = "overflow")]
fn test_exp_overflow_panics() {
    let _ = StrictlyPositive::THOUSAND.exp();
}

#[test]
fn test_powi_positive_and_negative_exponents() {
    assert_eq!(StrictlyPositive::TWO.powi(3), StrictlyPositive::EIGHT);
    assert_eq!(StrictlyPositive::TWO.checked_powi(-1), Ok(sp(dec!(0.5))));
}

#[test]
fn test_checked_powi_underflow_returns_out_of_bounds() {
    assert!(matches!(
        StrictlyPositive::MIN.checked_powi(2),
        Err(PositiveError::OutOfBounds { .. })
    ));
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in powi")]
fn test_powi_underflow_panics() {
    let _ = StrictlyPositive::MIN.powi(2);
}

#[test]
fn test_checked_powi_overflow_returns_arithmetic_error() {
    assert!(matches!(
        StrictlyPositive::MAX.checked_powi(2),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
fn test_powu_returns_strict() {
    assert_eq!(StrictlyPositive::TWO.powu(3), StrictlyPositive::EIGHT);
    assert_eq!(
        StrictlyPositive::TWO.checked_powu(0),
        Ok(StrictlyPositive::ONE)
    );
    assert!(matches!(
        StrictlyPositive::MAX.checked_powu(2),
        Err(PositiveError::ArithmeticError { .. })
    ));
}

#[test]
fn test_pow_returns_strict() {
    assert_eq!(
        StrictlyPositive::TWO.pow(Positive::THREE),
        StrictlyPositive::EIGHT
    );
    assert_eq!(
        StrictlyPositive::TWO.checked_pow(Positive::THREE),
        Ok(StrictlyPositive::EIGHT)
    );
}

#[test]
fn test_floor_ok_and_below_one() {
    assert_eq!(sp(dec!(1.9)).floor(), StrictlyPositive::ONE);
    assert_eq!(sp(dec!(0.3)).checked_floor(), Err(strict_oob(dec!(0))));
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in floor")]
fn test_floor_below_one_panics() {
    let _ = sp(dec!(0.3)).floor();
}

#[test]
fn test_ceiling_is_at_least_one() {
    assert_eq!(sp(dec!(0.3)).ceiling(), StrictlyPositive::ONE);
    assert_eq!(
        StrictlyPositive::MIN.checked_ceiling(),
        Ok(StrictlyPositive::ONE)
    );
    assert_eq!(sp(dec!(1.1)).checked_ceiling(), Ok(StrictlyPositive::TWO));
    assert_eq!(
        StrictlyPositive::MAX.checked_ceiling(),
        Ok(StrictlyPositive::MAX)
    );
}

#[test]
fn test_round_ok_and_to_zero() {
    assert_eq!(sp(dec!(1.6)).round(), StrictlyPositive::TWO);
    assert_eq!(sp(dec!(0.3)).checked_round(), Err(strict_oob(dec!(0))));
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in round")]
fn test_round_to_zero_panics() {
    let _ = sp(dec!(0.3)).round();
}

#[test]
fn test_round_to_ok_and_to_zero() {
    assert_eq!(sp(dec!(1.2345)).round_to(2), sp(dec!(1.23)));
    assert_eq!(
        sp(dec!(0.004)).checked_round_to(2),
        Err(strict_oob(dec!(0)))
    );
    assert_eq!(sp(dec!(0.004)).checked_round_to(3), Ok(sp(dec!(0.004))));
}

#[test]
fn test_checked_round_to_invalid_precision_returns_invalid_precision() {
    assert!(matches!(
        StrictlyPositive::ONE.checked_round_to(29),
        Err(PositiveError::InvalidPrecision { precision: 29, .. })
    ));
}

#[test]
#[should_panic(expected = "precision 29 is invalid")]
fn test_round_to_invalid_precision_panics() {
    let _ = StrictlyPositive::ONE.round_to(29);
}

#[test]
#[should_panic(expected = "StrictlyPositive invariant broken in round_to")]
fn test_round_to_zero_at_scale_panics() {
    let _ = sp(dec!(0.004)).round_to(2);
}

#[test]
fn test_max_min() {
    assert_eq!(
        StrictlyPositive::ONE.max(StrictlyPositive::TWO),
        StrictlyPositive::TWO
    );
    assert_eq!(
        StrictlyPositive::TWO.max(StrictlyPositive::ONE),
        StrictlyPositive::TWO
    );
    assert_eq!(
        StrictlyPositive::ONE.min(StrictlyPositive::TWO),
        StrictlyPositive::ONE
    );
    assert_eq!(
        StrictlyPositive::TWO.min(StrictlyPositive::ONE),
        StrictlyPositive::ONE
    );
}

#[test]
fn test_clamp_within_below_above() {
    let (lo, hi) = (StrictlyPositive::TWO, StrictlyPositive::FIVE);
    assert_eq!(StrictlyPositive::ONE.clamp(lo, hi), lo);
    assert_eq!(StrictlyPositive::TEN.clamp(lo, hi), hi);
    assert_eq!(
        StrictlyPositive::THREE.clamp(lo, hi),
        StrictlyPositive::THREE
    );
    assert_eq!(StrictlyPositive::THREE.checked_clamp(lo, lo), Ok(lo));
}

#[test]
fn test_checked_clamp_inverted_returns_out_of_bounds() {
    assert_eq!(
        StrictlyPositive::THREE.checked_clamp(StrictlyPositive::FIVE, StrictlyPositive::TWO),
        Err(PositiveError::OutOfBounds {
            value: dec!(5),
            min: dec!(1e-28),
            max: dec!(2),
        })
    );
}

#[test]
#[should_panic(expected = "clamp range is inverted")]
fn test_clamp_inverted_panics() {
    let _ = StrictlyPositive::THREE.clamp(StrictlyPositive::FIVE, StrictlyPositive::TWO);
}

// ---------------------------------------------------------------------------
// Invariant sweep: no operation may yield a value <= 0
// ---------------------------------------------------------------------------

#[test]
fn test_invariant_holds_for_every_operation_over_a_value_grid() {
    let grid = [
        dec!(1e-28),
        dec!(1e-20),
        dec!(1e-14),
        dec!(0.004),
        dec!(0.3),
        dec!(0.5),
        dec!(1),
        dec!(1.5),
        dec!(3),
        dec!(1000),
        dec!(1e14),
        dec!(1e20),
        Decimal::MAX,
    ];
    let values: Vec<StrictlyPositive> = grid.iter().map(|d| sp(*d)).collect();
    let check = |r: Result<StrictlyPositive, PositiveError>| {
        if let Ok(v) = r {
            assert!(v.to_dec() > Decimal::ZERO, "invariant broken: {v}");
        }
    };
    for &a in &values {
        check(a.checked_sqrt());
        check(a.checked_floor());
        check(a.checked_ceiling());
        check(a.checked_round());
        check(a.checked_round_to(2));
        check(a.checked_powi(-2));
        check(a.checked_powi(3));
        check(a.checked_powu(2));
        if a.to_dec() < dec!(50) {
            check(a.checked_exp());
        }
        for &b in &values {
            check(a.checked_add(&b));
            check(a.checked_sub(&b));
            check(a.checked_mul(&b));
            check(a.checked_div(&b));
            check(a.checked_add_positive(&b.to_positive()));
            check(StrictlyPositive::checked_sum([a, b]));
            check(a.checked_clamp(a.min(b), a.max(b)));
        }
    }
}
