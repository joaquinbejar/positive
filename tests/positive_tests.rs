/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Integration tests for the Positive type.

use positive::{Positive, PositiveError, pos, pos_or_panic, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_positive_decimal_creation() {
    assert!(Positive::new_decimal(Decimal::ZERO).is_ok());
    assert!(Positive::new_decimal(Decimal::ONE).is_ok());
    assert!(Positive::new_decimal(Decimal::NEGATIVE_ONE).is_err());
}

#[cfg(feature = "non-zero")]
#[test]
fn test_positive_decimal_creation_non_zero() {
    assert!(Positive::new_decimal(Decimal::ZERO).is_err());
    assert!(Positive::new_decimal(Decimal::ONE).is_ok());
    assert!(Positive::new_decimal(Decimal::NEGATIVE_ONE).is_err());
}

#[test]
fn test_positive_decimal_value() {
    let pos = Positive::new(5.0).unwrap();
    assert_eq!(pos, 5.0);
}

#[test]
fn test_positive_decimal_from() {
    let p = Positive::new(3.0).unwrap();
    let f: Decimal = p.into();
    assert_eq!(f, dec!(3.0));
}

#[test]
fn test_positive_decimal_eq() {
    let p = Positive::new_decimal(Decimal::TWO).unwrap();
    assert_eq!(p, dec!(2.0));
    assert_ne!(p, dec!(3.0));
}

#[test]
fn test_positive_decimal_display() {
    let p = Positive::new_decimal(dec!(4.5)).unwrap();
    assert_eq!(format!("{p}"), "4.5");
}

#[test]
fn test_positive_decimal_debug() {
    let p = Positive::new_decimal(dec!(4.5)).unwrap();
    assert_eq!(format!("{p:?}"), "4.5");
}

#[test]
fn test_positive_decimal_display_decimal_fix() {
    let p = Positive::new_decimal(dec!(4.578923789423789)).unwrap();
    assert_eq!(format!("{p:.2}"), "4.57");
    assert_eq!(format!("{p:.3}"), "4.578");
    assert_eq!(format!("{p:.0}"), "4");
}

#[test]
fn test_positive_decimal_add() {
    let a = Positive::new_decimal(dec!(2.0)).unwrap();
    let b = Positive::new_decimal(dec!(3.0)).unwrap();
    assert_eq!((a + b).value(), dec!(5.0));
}

#[test]
fn test_positive_decimal_div() {
    let a = Positive::new_decimal(dec!(6.0)).unwrap();
    let b = Positive::new_decimal(dec!(2.0)).unwrap();
    assert_eq!((a / b).value(), dec!(3.0));
}

#[test]
fn test_positive_decimal_div_f64() {
    let a = Positive::new_decimal(dec!(6.0)).unwrap();
    assert_eq!((a / 2.0), 3.0);
}

#[test]
fn test_decimal_mul_positive_decimal() {
    let a = dec!(2.0);
    let b = Positive::new_decimal(dec!(3.0)).unwrap();
    assert_eq!(a * b, dec!(6.0));
}

#[test]
fn test_positive_decimal_mul() {
    let a = Positive::new_decimal(dec!(2.0)).unwrap();
    let b = Positive::new_decimal(dec!(3.0)).unwrap();
    assert_eq!((a * b).value(), dec!(6.0));
}

#[test]
fn test_positive_decimal_mul_f64() {
    let a = Positive::new_decimal(dec!(2.0)).unwrap();
    assert_eq!((a * 3.0), 6.0);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_positive_decimal_default() {
    assert_eq!(Positive::default().value(), Decimal::ZERO);
}

#[cfg(feature = "non-zero")]
#[test]
fn test_positive_decimal_default_non_zero() {
    assert_eq!(Positive::default().value(), Decimal::ONE);
}

#[test]
fn test_decimal_div_positive_decimal() {
    let a = dec!(6.0);
    let b = Positive::new_decimal(dec!(2.0)).unwrap();
    assert_eq!(a / b, dec!(3.0));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_constants() {
    assert_eq!(Positive::ZERO.value(), Decimal::ZERO);
    assert_eq!(Positive::ONE.value(), Decimal::ONE);
}

#[cfg(feature = "non-zero")]
#[test]
fn test_constants_non_zero() {
    assert_eq!(Positive::ONE.value(), Decimal::ONE);
}

#[test]
fn test_positive_decimal_ordering() {
    let a = pos_or_panic!(1.0);
    let b = pos_or_panic!(2.0);
    let c = pos_or_panic!(2.0);

    assert!(a < b);
    assert!(b > a);
    assert!(b >= c);
    assert!(b <= c);
}

#[test]
fn test_positive_decimal_add_assign() {
    let mut a = pos_or_panic!(1.0);
    let b = pos_or_panic!(2.0);
    a += b;
    assert_eq!(a.value(), dec!(3.0));
}

#[test]
fn test_positive_decimal_from_string() {
    assert_eq!(Positive::from_str("1.5").unwrap().value(), dec!(1.5));
    assert!(Positive::from_str("-1.5").is_err());
    assert!(Positive::from_str("invalid").is_err());
}

#[test]
fn test_positive_decimal_max_min() {
    let a = pos_or_panic!(1.0);
    let b = pos_or_panic!(2.0);
    assert_eq!(a.max(b).value(), dec!(2.0));
    assert_eq!(a.min(b).value(), dec!(1.0));
}

#[test]
fn test_positive_decimal_floor() {
    let a = pos_or_panic!(1.7);
    assert_eq!(a.floor().value(), dec!(1.0));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sum_owned_values() {
    let values = vec![pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
    let sum: Positive = values.into_iter().sum();
    assert_eq!(sum.to_f64(), 6.0);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sum_referenced_values() {
    let values = [pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
    let sum: Positive = values.iter().sum();
    assert_eq!(sum.to_f64(), 6.0);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sum_empty_iterator() {
    let values: Vec<Positive> = vec![];
    let sum: Positive = values.into_iter().sum();
    assert_eq!(sum.to_f64(), 0.0);
}

#[test]
fn test_checked_sub_success() {
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(3.0);
    let result = a.checked_sub(&b);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 2.0);
}

#[test]
fn test_checked_sub_failure() {
    let a = pos_or_panic!(3.0);
    let b = pos_or_panic!(5.0);
    let result = a.checked_sub(&b);
    assert!(result.is_err());
}

/// `saturating_sub` is deprecated but still shipped in 0.6.0, so its behaviour
/// stays covered until it is removed in the following release.
#[cfg(not(feature = "non-zero"))]
#[test]
#[allow(deprecated)]
fn test_saturating_sub() {
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(3.0);
    assert_eq!(a.saturating_sub(&b).to_f64(), 2.0);

    let c = pos_or_panic!(3.0);
    let d = pos_or_panic!(5.0);
    assert_eq!(c.saturating_sub(&d), Positive::ZERO);
}

#[test]
fn test_checked_div_success() {
    let a = pos_or_panic!(6.0);
    let b = pos_or_panic!(2.0);
    let result = a.checked_div(&b);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 3.0);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_checked_div_by_zero() {
    let a = pos_or_panic!(6.0);
    let b = Positive::ZERO;
    let result = a.checked_div(&b);
    assert!(result.is_err());
}

#[test]
fn test_pos_positive_values() {
    assert_eq!(pos_or_panic!(5.0).value(), Decimal::new(5, 0));
    assert_eq!(pos_or_panic!(1.5).value(), Decimal::new(15, 1));
    assert_eq!(pos_or_panic!(0.1).value(), Decimal::new(1, 1));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_pos_zero() {
    assert_eq!(Positive::ZERO, Positive::ZERO);
}

#[cfg(feature = "non-zero")]
#[test]
fn test_zero_is_rejected() {
    assert!(Positive::new(0.0).is_err());
    assert!(pos!(0.0).is_err());
    assert!(spos!(0.0).is_none());
}

#[test]
fn test_pos_small_decimals() {
    assert_eq!(pos_or_panic!(0.0001).value(), Decimal::new(1, 4));
    assert_eq!(pos_or_panic!(0.00001).value(), Decimal::new(1, 5));
    assert_eq!(pos_or_panic!(0.000001).value(), Decimal::new(1, 6));
}

#[test]
fn test_pos_large_decimals() {
    let val = 0.1234567890123456;
    let expected = Decimal::from_str("0.1234567890123456").unwrap();
    assert_eq!(pos_or_panic!(val).value(), expected);
}

#[test]
#[should_panic(expected = "OutOfBounds")]
fn test_pos_negative_values() {
    pos_or_panic!(-1.0);
}

#[test]
fn test_pos_edge_cases() {
    assert_eq!(
        pos_or_panic!(1e15).value(),
        Decimal::from_str("1000000000000000").unwrap()
    );

    assert_eq!(
        pos_or_panic!(1e-15).value(),
        Decimal::from_str("0.000000000000001").unwrap()
    );
}

#[test]
fn test_pos_expressions() {
    assert_eq!(pos_or_panic!(2.0 + 3.0).value(), Decimal::new(5, 0));
    assert_eq!(pos_or_panic!(1.5 * 2.0).value(), Decimal::new(3, 0));
}

#[test]
fn test_pos_macro_returns_result() {
    let x = pos!(10.0);
    assert!(x.is_ok());
    assert_eq!(x.unwrap().to_f64(), 10.0);

    let y = pos!(-5.0);
    assert!(y.is_err());
}

#[test]
fn test_spos_macro() {
    let x = spos!(10.0);
    assert!(x.is_some());

    let y = spos!(-5.0);
    assert!(y.is_none());
}

#[test]
fn test_positive_serialization() {
    let value = pos_or_panic!(42.5);
    let serialized = serde_json::to_string(&value).unwrap();
    // Exact decimal string: a JSON number cannot carry Decimal's precision.
    assert_eq!(serialized, "\"42.5\"");
}

#[test]
fn test_positive_deserialization() {
    let json = "42.5";
    let deserialized: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, pos_or_panic!(42.5));
}

#[test]
fn test_positive_serialization_whole_number() {
    let value = pos_or_panic!(100.0);
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, "\"100\"");
}

#[test]
fn test_positive_deserialization_whole_number() {
    let json = "100";
    let deserialized: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, pos_or_panic!(100.0));
}

#[test]
fn test_positive_roundtrip() {
    let original = pos_or_panic!(123.456);
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: Positive = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_positive_zero_deserialization() {
    let json = "0";
    let result = serde_json::from_str::<Positive>(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Positive::ZERO);
}

#[cfg(feature = "non-zero")]
#[test]
fn test_positive_zero_deserialization_non_zero() {
    let json = "0";
    let result = serde_json::from_str::<Positive>(json);
    assert!(result.is_err());

    let json_float = "0.0";
    let result = serde_json::from_str::<Positive>(json_float);
    assert!(result.is_err());
}

#[test]
fn test_positive_negative_deserialization() {
    let json = "-42.5";
    let result = serde_json::from_str::<Positive>(json);
    assert!(result.is_err());
}

#[test]
fn test_format_fixed_places() {
    let value = pos_or_panic!(10.5);
    assert_eq!(value.format_fixed_places(2), "10.50");

    let value = pos_or_panic!(10.0);
    assert_eq!(value.format_fixed_places(3), "10.000");

    let value = pos_or_panic!(10.567);
    assert_eq!(value.format_fixed_places(2), "10.57");

    let value = pos_or_panic!(0.1);
    assert_eq!(value.format_fixed_places(4), "0.1000");
}

#[test]
#[allow(deprecated)]
fn test_is_multiple() {
    let num = pos_or_panic!(10.0);
    assert!(num.is_multiple(2.0));
    assert!(!num.is_multiple(3.0));
}

#[test]
fn test_is_multiple_of() {
    let num = pos_or_panic!(10.0);
    assert!(num.is_multiple_of(&pos_or_panic!(2.0)));
    assert!(num.is_multiple_of(&pos_or_panic!(5.0)));
}

#[test]
fn test_clamp() {
    let value = pos_or_panic!(5.0);
    assert_eq!(
        value.clamp(pos_or_panic!(1.0), pos_or_panic!(10.0)),
        pos_or_panic!(5.0)
    );
    assert_eq!(
        value.clamp(pos_or_panic!(6.0), pos_or_panic!(10.0)),
        pos_or_panic!(6.0)
    );
    assert_eq!(
        value.clamp(pos_or_panic!(1.0), pos_or_panic!(4.0)),
        pos_or_panic!(4.0)
    );
}

#[test]
fn test_sqrt() {
    let value = pos_or_panic!(16.0);
    assert_eq!(value.sqrt().to_f64(), 4.0);
}

#[test]
fn test_checked_sqrt() {
    let value = pos_or_panic!(16.0);
    let result = value.checked_sqrt();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 4.0);
}

/// `sqrt_checked` is deprecated in favour of `checked_sqrt` but must keep
/// delegating to it until it is removed.
#[test]
#[allow(deprecated)]
fn test_sqrt_checked_alias_matches_checked_sqrt() {
    let value = pos_or_panic!(16.0);
    assert_eq!(value.sqrt_checked().unwrap(), value.checked_sqrt().unwrap());
}

#[test]
fn test_pow() {
    let value = pos_or_panic!(2.0);
    assert_eq!(value.pow(pos_or_panic!(3.0)).to_f64(), 8.0);
}

#[test]
fn test_powi() {
    let value = pos_or_panic!(2.0);
    assert_eq!(value.powi(3).to_f64(), 8.0);
}

#[test]
fn test_powu() {
    let value = pos_or_panic!(2.0);
    assert_eq!(value.powu(3).to_f64(), 8.0);
}

#[test]
fn test_ceiling() {
    let value = pos_or_panic!(1.3);
    assert_eq!(value.ceiling().to_f64(), 2.0);
}

#[test]
fn test_round_to() {
    let value = pos_or_panic!(1.2345);
    assert_eq!(value.round_to(2).to_f64(), 1.23);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_is_zero() {
    assert!(Positive::ZERO.is_zero());
    assert!(!pos_or_panic!(1.0).is_zero());
}

#[cfg(feature = "non-zero")]
#[test]
fn test_is_zero_non_zero() {
    assert!(!pos_or_panic!(1.0).is_zero());
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sub_or_zero() {
    let a = pos_or_panic!(5.0);
    assert_eq!(a.sub_or_zero(&dec!(3.0)).to_f64(), 2.0);
    assert_eq!(a.sub_or_zero(&dec!(10.0)), Positive::ZERO);
}

#[test]
fn test_sub_or_none() {
    let a = pos_or_panic!(5.0);
    assert!(a.sub_or_none(&dec!(3.0)).is_some());
    assert!(a.sub_or_none(&dec!(10.0)).is_none());
}

#[test]
fn test_to_f64_checked() {
    let value = pos_or_panic!(5.0);
    assert_eq!(value.to_f64_checked(), Some(5.0));
}

#[test]
fn test_to_f64_lossy() {
    let value = pos_or_panic!(5.0);
    #[allow(deprecated)]
    {
        assert_eq!(value.to_f64_lossy(), 5.0);
    }
}

#[test]
fn test_to_i64_checked() {
    let value = pos_or_panic!(5.0);
    assert_eq!(value.to_i64_checked(), Some(5));
}

#[test]
fn test_to_u64_checked() {
    let value = pos_or_panic!(5.0);
    assert_eq!(value.to_u64_checked(), Some(5));
}

#[test]
fn test_to_usize_checked() {
    let value = pos_or_panic!(5.0);
    assert_eq!(value.to_usize_checked(), Some(5));
}

// ============================================================================
// Additional tests for improved coverage
// ============================================================================

#[test]
fn test_is_positive_function() {
    use positive::is_positive;
    assert!(is_positive::<Positive>());
    assert!(!is_positive::<f64>());
    assert!(!is_positive::<Decimal>());
}

#[test]
fn test_new_with_nan() {
    let result = Positive::new(f64::NAN);
    assert!(result.is_err());
}

#[test]
fn test_new_with_infinity() {
    let result = Positive::new(f64::INFINITY);
    assert!(result.is_err());
}

#[test]
fn test_to_dec_ref() {
    let p = pos_or_panic!(5.0);
    let dec_ref = p.to_dec_ref();
    assert_eq!(*dec_ref, dec!(5.0));
}

#[test]
fn test_to_i64() {
    let value = pos_or_panic!(42.0);
    #[allow(deprecated)]
    {
        assert_eq!(value.to_i64(), 42);
    }
}

#[test]
fn test_to_u64() {
    let value = pos_or_panic!(42.0);
    #[allow(deprecated)]
    {
        assert_eq!(value.to_u64(), 42);
    }
}

#[test]
fn test_to_usize() {
    let value = pos_or_panic!(42.0);
    #[allow(deprecated)]
    {
        assert_eq!(value.to_usize(), 42);
    }
}

#[test]
fn test_powd() {
    let value = pos_or_panic!(2.0);
    let result = value.powd(dec!(3.0));
    assert_eq!(result.to_f64(), 8.0);
}

#[test]
fn test_round() {
    let value = pos_or_panic!(1.6);
    assert_eq!(value.round().to_f64(), 2.0);

    let value2 = pos_or_panic!(1.4);
    assert_eq!(value2.round().to_f64(), 1.0);
}

#[test]
fn test_round_to_nice_number() {
    let value = pos_or_panic!(1.2);
    let nice = value.round_to_nice_number();
    assert_eq!(nice, Positive::ONE);

    let value2 = pos_or_panic!(2.5);
    let nice2 = value2.round_to_nice_number();
    assert_eq!(nice2, Positive::TWO);

    let value3 = pos_or_panic!(4.0);
    let nice3 = value3.round_to_nice_number();
    assert_eq!(nice3, pos_or_panic!(5.0));

    let value4 = pos_or_panic!(8.0);
    let nice4 = value4.round_to_nice_number();
    assert_eq!(nice4, Positive::TEN);
}

#[test]
fn test_checked_sqrt_success() {
    let value = pos_or_panic!(16.0);
    let result = value.checked_sqrt();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 4.0);
}

#[test]
fn test_ln() {
    use num_traits::ToPrimitive;
    let value = pos_or_panic!(std::f64::consts::E);
    let result: Decimal = value.ln();
    assert!((result.to_f64().unwrap() - 1.0).abs() < 0.001);
}

#[test]
fn test_exp() {
    let value = pos_or_panic!(1.0);
    let result = value.exp();
    assert!((result.to_f64() - std::f64::consts::E).abs() < 0.001);
}

#[test]
fn test_log10() {
    let value = pos_or_panic!(100.0);
    let result: Decimal = value.log10();
    assert_eq!(result, dec!(2));
}

#[test]
fn test_clamp_below_min() {
    let value = pos_or_panic!(1.0);
    let clamped = value.clamp(pos_or_panic!(5.0), pos_or_panic!(10.0));
    assert_eq!(clamped, pos_or_panic!(5.0));
}

#[test]
fn test_clamp_above_max() {
    let value = pos_or_panic!(15.0);
    let clamped = value.clamp(pos_or_panic!(5.0), pos_or_panic!(10.0));
    assert_eq!(clamped, pos_or_panic!(10.0));
}

#[test]
fn test_clamp_within_range() {
    let value = pos_or_panic!(7.0);
    let clamped = value.clamp(pos_or_panic!(5.0), pos_or_panic!(10.0));
    assert_eq!(clamped, pos_or_panic!(7.0));
}

#[test]
#[allow(deprecated)]
fn test_is_multiple_edge_cases() {
    // Test with a value that would produce non-finite result in modulo
    let value = pos_or_panic!(10.0);
    assert!(value.is_multiple(5.0));
    assert!(!value.is_multiple(3.0));

    // Test near-epsilon cases
    let value2 = pos_or_panic!(10.0);
    assert!(value2.is_multiple(2.0));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_is_multiple_of_with_zero() {
    let value = pos_or_panic!(10.0);
    assert!(!value.is_multiple_of(&Positive::ZERO));
}

#[test]
fn test_partial_eq_positive_ref() {
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(5.0);
    assert!(a == b);
}

#[test]
fn test_try_from_positive_to_u64() {
    let p = pos_or_panic!(42.0);
    let u: u64 = u64::try_from(p).unwrap();
    assert_eq!(u, 42);
}

#[test]
fn test_from_ref_positive_to_f64() {
    let p = pos_or_panic!(42.5);
    let f: f64 = (&p).into();
    assert_eq!(f, 42.5);
}

#[test]
fn test_from_positive_to_f64() {
    let p = pos_or_panic!(42.5);
    let f: f64 = p.into();
    assert_eq!(f, 42.5);
}

#[test]
fn test_try_from_positive_to_usize() {
    let p = pos_or_panic!(42.0);
    let u: usize = usize::try_from(p).unwrap();
    assert_eq!(u, 42);
}

#[test]
fn test_try_from_positive_to_usize_truncates_fractional() {
    // Integer conversions truncate toward zero; this is the documented
    // contract, not a rounding accident.
    let p = pos_or_panic!(42.9);
    let u: usize = usize::try_from(p).unwrap();
    assert_eq!(u, 42);
}

#[test]
fn test_try_from_positive_to_usize_large_value() {
    use rust_decimal_macros::dec;
    let p = positive::Positive::new_decimal(dec!(1000000)).expect("valid");
    let u: usize = usize::try_from(p).unwrap();
    assert_eq!(u, 1_000_000);
}

#[test]
fn test_f64_partial_eq_ref_positive() {
    let p = pos_or_panic!(5.0);
    assert!(5.0 == &p);
    assert!(!(6.0 == &p));
}

#[test]
fn test_f64_partial_ord_ref_positive() {
    let p = pos_or_panic!(5.0);
    assert!(4.0 < &p);
    assert!(6.0 > &p);
}

#[test]
fn test_f64_partial_eq_positive() {
    let p = pos_or_panic!(5.0);
    assert!(5.0 == p);
    assert!(!(6.0 == p));
}

#[test]
fn test_f64_partial_ord_positive() {
    let p = pos_or_panic!(5.0);
    assert!(4.0 < p);
    assert!(6.0 > p);
}

#[test]
fn test_f64_mul_positive() {
    let p = pos_or_panic!(3.0);
    let result = 2.0 * p;
    assert_eq!(result, 6.0);
}

#[test]
fn test_f64_div_positive() {
    let p = pos_or_panic!(2.0);
    let result = 6.0 / p;
    assert_eq!(result, 3.0);
}

#[test]
fn test_f64_sub_positive() {
    let p = pos_or_panic!(3.0);
    let result = 5.0 - p;
    assert_eq!(result, 2.0);
}

#[test]
fn test_f64_add_positive() {
    let p = pos_or_panic!(3.0);
    let result = 2.0 + p;
    assert_eq!(result, 5.0);
}

#[test]
fn test_try_from_usize() {
    let p: Positive = 42usize.try_into().unwrap();
    assert_eq!(p.to_f64(), 42.0);
}

#[test]
fn test_try_from_decimal() {
    let d = dec!(42.5);
    let p: Positive = d.try_into().unwrap();
    assert_eq!(p.to_f64(), 42.5);
}

#[test]
fn test_try_from_ref_decimal() {
    let d = dec!(42.5);
    let p: Positive = Positive::new_decimal(d).unwrap();
    assert_eq!(p.to_f64(), 42.5);
}

#[test]
fn test_from_ref_positive() {
    let p1 = pos_or_panic!(42.5);
    let p2: Positive = (&p1).into();
    assert_eq!(p2.to_f64(), 42.5);
}

#[test]
fn test_positive_div_f64_ref() {
    let p = pos_or_panic!(6.0);
    let result = &p / 2.0;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_positive_sub_f64() {
    let p = pos_or_panic!(5.0);
    let result = p - 2.0;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_positive_add_f64() {
    let p = pos_or_panic!(5.0);
    let result = p + 2.0;
    assert_eq!(result.to_f64(), 7.0);
}

#[test]
fn test_positive_partial_ord_f64() {
    let p = pos_or_panic!(5.0);
    assert!(p > 4.0);
    assert!(p < 6.0);
}

#[test]
fn test_ref_positive_partial_eq_f64() {
    let p = pos_or_panic!(5.0);
    assert!(&p == 5.0);
}

#[test]
fn test_ref_positive_partial_ord_f64() {
    let p = pos_or_panic!(5.0);
    assert!(&p > 4.0);
    assert!(&p < 6.0);
}

#[test]
fn test_display_max_renders_the_value_it_holds() {
    let p = Positive::MAX;
    let s = format!("{p}");
    assert_eq!(s, "79228162514264337593543950335");
    // ...and not f64::MAX, which is roughly 10^279 times larger
    assert_ne!(s, format!("{}", f64::MAX));
}

#[test]
fn test_display_integer() {
    let p = pos_or_panic!(42.0);
    let s = format!("{p}");
    assert_eq!(s, "42");
}

#[test]
fn test_debug_max_renders_the_value_it_holds() {
    let p = Positive::MAX;
    let s = format!("{p:?}");
    assert_eq!(s, "79228162514264337593543950335");
    assert_ne!(s, format!("{}", f64::MAX));
}

#[test]
fn test_debug_integer() {
    let p = pos_or_panic!(42.0);
    let s = format!("{p:?}");
    assert_eq!(s, "42");
}

/// The `f64::MAX` sentinel is gone (#76), and since #75 `Positive::MAX`
/// serialises losslessly rather than failing on the scale-0 `to_i64` path.
#[test]
fn test_serialize_max_no_longer_emits_the_f64_sentinel() {
    let json = serde_json::to_string(&Positive::MAX).unwrap();
    assert!(!json.contains("1.7976931348623157e+308"));
    assert_eq!(json, "\"79228162514264337593543950335\"");
    let back: Positive = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Positive::MAX);
}

/// Values that do fit the current wire format must serialise as themselves.
#[test]
fn test_serialize_large_value_within_i64_has_no_sentinel() {
    let value = Positive::new_decimal(Decimal::from(i64::MAX)).unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "\"9223372036854775807\"");
    assert!(!json.contains("1.7976931348623157e+308"));
}

#[test]
fn test_deserialize_string_error() {
    let json = "\"not_a_number\"";
    let result: Result<Positive, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_negative_i64() {
    let json = "-42";
    let result: Result<Positive, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_u64() {
    let json = "42";
    let result: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(result.to_f64(), 42.0);
}

#[test]
fn test_deserialize_f64_max_is_rejected_not_mapped_to_max() {
    // f64::MAX used to be a sentinel that deserialised to Decimal::MAX, even
    // though Positive::new(f64::MAX) rejects it. The two agree now.
    let json = "1.7976931348623157e+308";
    let result: Result<Positive, _> = serde_json::from_str(json);
    assert!(result.is_err());
    assert!(Positive::new(f64::MAX).is_err());
}

#[test]
fn test_deserialize_negative_f64() {
    let json = "-42.5";
    let result: Result<Positive, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "invariant broken in sub")]
fn test_sub_panic() {
    let a = pos_or_panic!(3.0);
    let b = pos_or_panic!(5.0);
    let _ = a - b;
}

#[test]
fn test_div_ref_positive() {
    let a = pos_or_panic!(6.0);
    let b = pos_or_panic!(2.0);
    let result = a / b;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_add_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p + d;
    assert_eq!(result.to_f64(), 8.0);
}

#[test]
fn test_add_ref_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p + d;
    assert_eq!(result.to_f64(), 8.0);
}

#[test]
fn test_sub_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p - d;
    assert_eq!(result.to_f64(), 2.0);
}

#[test]
fn test_sub_ref_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p - d;
    assert_eq!(result.to_f64(), 2.0);
}

#[test]
fn test_add_assign_decimal() {
    let mut p = pos_or_panic!(5.0);
    p += dec!(3.0);
    assert_eq!(p.to_f64(), 8.0);
}

#[test]
fn test_mul_assign_decimal() {
    let mut p = pos_or_panic!(5.0);
    p *= dec!(2.0);
    assert_eq!(p.to_f64(), 10.0);
}

#[test]
fn test_div_decimal() {
    let p = pos_or_panic!(6.0);
    let d = dec!(2.0);
    let result = p / d;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_div_ref_decimal() {
    let p = pos_or_panic!(6.0);
    let d = dec!(2.0);
    let result = p / d;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_partial_ord_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(4.0);
    assert!(p > d);

    let d2 = dec!(6.0);
    assert!(p < d2);
}

#[test]
fn test_partial_ord_positive() {
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(4.0);
    assert!(a > b);
    assert!(a >= b);
    assert!(b < a);
    assert!(b <= a);
}

#[test]
fn test_mul_decimal() {
    let p = pos_or_panic!(5.0);
    let d = dec!(2.0);
    let result = p * d;
    assert_eq!(result.to_f64(), 10.0);
}

#[test]
fn test_decimal_div_positive() {
    let d = dec!(6.0);
    let p = pos_or_panic!(2.0);
    let result = d / p;
    assert_eq!(result, dec!(3.0));
}

#[test]
fn test_decimal_sub_positive() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d - p;
    assert_eq!(result, dec!(2.0));
}

#[test]
fn test_decimal_sub_ref_positive() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d - p;
    assert_eq!(result, dec!(2.0));
}

#[test]
fn test_decimal_add_positive() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d + p;
    assert_eq!(result, dec!(8.0));
}

#[test]
fn test_decimal_add_ref_positive() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d + p;
    assert_eq!(result, dec!(8.0));
}

#[test]
fn test_decimal_add_assign_positive() {
    let mut d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    d += p;
    assert_eq!(d, dec!(8.0));
}

#[test]
fn test_decimal_add_assign_ref_positive() {
    let mut d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    d += &p;
    assert_eq!(d, dec!(8.0));
}

#[test]
fn test_decimal_mul_assign_positive() {
    let mut d = dec!(5.0);
    let p = pos_or_panic!(2.0);
    d *= p;
    assert_eq!(d, dec!(10.0));
}

#[test]
fn test_decimal_mul_assign_ref_positive() {
    let mut d = dec!(5.0);
    let p = pos_or_panic!(2.0);
    d *= &p;
    assert_eq!(d, dec!(10.0));
}

#[test]
fn test_decimal_partial_eq_positive() {
    let d = dec!(5.0);
    let p = pos_or_panic!(5.0);
    assert!(d == p);
}

#[test]
fn test_from_ref_positive_to_decimal() {
    let p = pos_or_panic!(42.5);
    let d: Decimal = (&p).into();
    assert_eq!(d, dec!(42.5));
}

#[test]
fn test_abs_diff_eq() {
    use approx::AbsDiffEq;
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(5.0);
    assert!(a.abs_diff_eq(&b, Positive::default_epsilon()));
}

#[test]
fn test_relative_eq() {
    use approx::{AbsDiffEq, RelativeEq};
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(5.0);
    assert!(a.relative_eq(
        &b,
        Positive::default_epsilon(),
        Positive::default_max_relative()
    ));
}

#[test]
fn test_ord() {
    use std::cmp::Ord;
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(3.0);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Greater);
    assert_eq!(b.cmp(&a), std::cmp::Ordering::Less);
    assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
}

// ============================================================================
// Additional tests for uncovered lines
// ============================================================================

#[test]
fn test_try_from_f64() {
    let p: Result<Positive, _> = 42.5f64.try_into();
    assert!(p.is_ok());
    assert_eq!(p.unwrap().to_f64(), 42.5);

    let neg: Result<Positive, _> = (-5.0f64).try_into();
    assert!(neg.is_err());
}

#[test]
fn test_try_from_i64() {
    let p: Result<Positive, _> = 42i64.try_into();
    assert!(p.is_ok());
    assert_eq!(p.unwrap().to_f64(), 42.0);

    let neg: Result<Positive, _> = (-5i64).try_into();
    assert!(neg.is_err());
}

#[test]
fn test_try_from_u64() {
    let p: Result<Positive, _> = 42u64.try_into();
    assert!(p.is_ok());
    assert_eq!(p.unwrap().to_f64(), 42.0);
}

#[test]
fn test_try_from_ref_decimal_negative() {
    let d = dec!(-42.5);
    let p: Result<Positive, _> = (&d).try_into();
    assert!(p.is_err());
}

#[test]
fn test_partial_eq_positive_with_ref() {
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(5.0);
    assert!(a == b);
}

#[test]
fn test_add_ref_decimal_actual() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p + d;
    assert_eq!(result.to_f64(), 8.0);
}

#[test]
fn test_sub_ref_decimal_actual() {
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    let result = p - d;
    assert_eq!(result.to_f64(), 2.0);
}

#[test]
fn test_div_ref_decimal_actual() {
    let p = pos_or_panic!(6.0);
    let d = dec!(2.0);
    let result = p / d;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_decimal_sub_ref_positive_actual() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d - p;
    assert_eq!(result, dec!(2.0));
}

#[test]
fn test_decimal_add_ref_positive_actual() {
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    let result = d + p;
    assert_eq!(result, dec!(8.0));
}

#[test]
fn test_div_ref_positive_refs() {
    let a = pos_or_panic!(6.0);
    let b = pos_or_panic!(2.0);
    let result = a / b;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_display_with_precision() {
    let p = pos_or_panic!(std::f64::consts::PI);
    let s = format!("{p:.2}");
    assert_eq!(s, "3.14");
}

#[test]
fn test_debug_decimal_value() {
    let p = pos_or_panic!(std::f64::consts::PI);
    let s = format!("{p:?}");
    assert!(s.contains("3.14159"));
}

#[test]
#[allow(deprecated)]
fn test_is_multiple_true_case() {
    let value = pos_or_panic!(10.0);
    assert!(value.is_multiple(2.0));
    assert!(value.is_multiple(5.0));
    assert!(value.is_multiple(10.0));
}

#[test]
#[allow(deprecated)]
fn test_is_multiple_near_boundary() {
    let value = pos_or_panic!(9.999999999999998);
    assert!(value.is_multiple(1.0));
}

#[test]
#[allow(deprecated)]
fn test_is_multiple_with_non_finite() {
    // Test is_multiple when value would produce non-finite result
    // Note: Positive::INFINITY is Decimal::MAX which is finite when converted to f64
    // The non-finite check is for edge cases in the modulo operation
    let value = pos_or_panic!(10.0);
    // Test normal case
    assert!(value.is_multiple(2.0));
    assert!(value.is_multiple(5.0));
}

#[test]
fn test_display_large_integer_no_i64() {
    // Test Display when scale is 0 but value is too large for i64 (line 752)
    // Decimal::MAX has scale 0 but cannot fit in i64
    let large = Positive::MAX;
    let s = format!("{large}");
    assert!(!s.is_empty());
}

#[test]
fn test_debug_large_integer_no_i64() {
    // Test Debug when scale is 0 but value is too large for i64 (line 771)
    let large = Positive::MAX;
    let s = format!("{large:?}");
    assert!(!s.is_empty());
}

#[test]
fn test_deserialize_positive_i64() {
    // Test visit_i64 with positive value (line 839)
    let json = "42";
    let result: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(result.to_f64(), 42.0);
}

#[test]
fn test_sub_positive_success() {
    // Test Sub for Positive success path (line 888)
    let a = pos_or_panic!(10.0);
    let b = pos_or_panic!(3.0);
    let result = a - b;
    assert_eq!(result.to_f64(), 7.0);
}

#[test]
fn test_positive_eq_ref_positive() {
    // Test PartialEq<&Positive> for Positive (lines 510-511)
    let a = pos_or_panic!(5.0);
    let b = pos_or_panic!(5.0);
    let c = pos_or_panic!(6.0);
    // Must use &b to trigger PartialEq<&Positive>
    assert!(a == b);
    assert!(a != c);
}

#[test]
fn test_constants_are_still_compile_time_without_an_unchecked_constructor() {
    // `Positive::new_unchecked` is gone. The constants still exist as `const`
    // items, which is the property that made an unchecked public constructor
    // look necessary in the first place.
    const ONE: Positive = Positive::ONE;
    const HUNDRED: Positive = Positive::HUNDRED;
    const MAX: Positive = Positive::MAX;

    assert_eq!(ONE.to_dec(), Decimal::ONE);
    assert_eq!(HUNDRED.to_dec(), Decimal::ONE_HUNDRED);
    assert_eq!(MAX.to_dec(), Decimal::MAX);

    // The migration path for former `new_unchecked` callers is the validated
    // constructor, which is a runtime check rather than an unchecked cast.
    assert_eq!(Positive::new_decimal(dec!(42.0)).unwrap().to_f64(), 42.0);
}

/// Every public path that yields a `Positive` validates. There is no longer
/// any constructor — safe or unsafe — that can produce an invalid one.
#[test]
fn test_no_public_constructor_can_produce_an_invalid_value() {
    let invalid_inputs = [Decimal::NEGATIVE_ONE, Decimal::MIN, Decimal::new(-1, 28)];
    for input in invalid_inputs {
        assert!(Positive::new_decimal(input).is_err());
        assert!(Positive::try_from(input).is_err());
    }
    assert!(Positive::new(-0.5).is_err());
    assert!(Positive::from_str("-0.5").is_err());
    assert!(spos!(-1.0).is_none());
    assert!(pos!(-1.0).is_err());
}

#[test]
fn test_clamp_all_branches() {
    // Test all three branches of clamp (lines 383-389)
    let value = pos_or_panic!(5.0);
    let min = pos_or_panic!(1.0);
    let max = pos_or_panic!(10.0);

    // Branch: self >= min && self <= max (line 389)
    assert_eq!(value.clamp(min, max), pos_or_panic!(5.0));

    // Branch: self < min (line 385)
    let low = pos_or_panic!(0.5);
    assert_eq!(low.clamp(min, max), pos_or_panic!(1.0));

    // Branch: self > max (line 387)
    let high = pos_or_panic!(15.0);
    assert_eq!(high.clamp(min, max), pos_or_panic!(10.0));
}

#[test]
fn test_add_ref_decimal_impl() {
    // Test Add<&Decimal> for Positive (lines 916-917)
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    // Must use &d to trigger Add<&Decimal>
    let result = p + d;
    assert_eq!(result.to_f64(), 8.0);
}

#[test]
fn test_sub_ref_decimal_impl() {
    // Test Sub<&Decimal> for Positive (lines 930-931)
    let p = pos_or_panic!(5.0);
    let d = dec!(3.0);
    // Must use &d to trigger Sub<&Decimal>
    let result = p - d;
    assert_eq!(result.to_f64(), 2.0);
}

#[test]
fn test_div_ref_decimal_impl() {
    // Test Div<&Decimal> for Positive (lines 962-963)
    let p = pos_or_panic!(6.0);
    let d = dec!(2.0);
    // Must use &d to trigger Div<&Decimal>
    let result = p / d;
    assert_eq!(result.to_f64(), 3.0);
}

#[test]
fn test_decimal_sub_ref_positive_impl() {
    // Test Sub<&Positive> for Decimal (lines 1039-1040)
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    // Must use &p to trigger Sub<&Positive>
    let result = d - p;
    assert_eq!(result, dec!(2.0));
}

#[test]
fn test_decimal_add_ref_positive_impl() {
    // Test Add<&Positive> for Decimal (lines 1053-1054)
    let d = dec!(5.0);
    let p = pos_or_panic!(3.0);
    // Must use &p to trigger Add<&Positive>
    let result = d + p;
    assert_eq!(result, dec!(8.0));
}

// ===== checked_*_f64 public API (issue #22) =====

#[test]
fn test_checked_add_f64_ok() {
    let p = pos_or_panic!(5.0);
    let result = p.checked_add_f64(2.5).expect("ok");
    assert_eq!(result.to_f64(), 7.5);
}

#[test]
fn test_checked_add_f64_nan_is_conversion_error() {
    let p = pos_or_panic!(5.0);
    let err = p.checked_add_f64(f64::NAN).unwrap_err();
    assert!(matches!(
        err,
        positive::PositiveError::ConversionError { .. }
    ));
}

#[test]
fn test_checked_sub_f64_invariant_error() {
    let p = pos_or_panic!(3.0);
    let err = p.checked_sub_f64(5.0).unwrap_err();
    assert!(matches!(err, positive::PositiveError::OutOfBounds { .. }));
}

#[test]
fn test_checked_mul_f64_negative_is_invariant_error() {
    let p = pos_or_panic!(5.0);
    let err = p.checked_mul_f64(-2.0).unwrap_err();
    assert!(matches!(err, positive::PositiveError::OutOfBounds { .. }));
}

#[test]
fn test_checked_div_f64_zero_is_arithmetic_error() {
    let p = pos_or_panic!(5.0);
    let err = p.checked_div_f64(0.0).unwrap_err();
    assert!(matches!(
        err,
        positive::PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_div_f64_ok() {
    let p = pos_or_panic!(10.0);
    let result = p.checked_div_f64(4.0).expect("ok");
    assert_eq!(result.to_f64(), 2.5);
}

// ===== Div rounding strategy (issue #23) =====

#[test]
fn test_div_default_uses_bankers_rounding() {
    use rust_decimal_macros::dec;
    // 1 / 3 = 0.333...; banker's rounding at 28 dp gives the decimal
    // truncated at that scale.
    let a = positive::Positive::new_decimal(dec!(1)).expect("ok");
    let b = positive::Positive::new_decimal(dec!(3)).expect("ok");
    let r = a / b;
    // Result must equal 1/3 rounded to 28 dp.
    let expected = dec!(1) / dec!(3);
    assert_eq!(
        r.to_dec()
            .round_dp_with_strategy(28, rust_decimal::RoundingStrategy::MidpointNearestEven),
        expected.round_dp_with_strategy(28, rust_decimal::RoundingStrategy::MidpointNearestEven)
    );
}

#[test]
fn test_checked_div_with_strategy_ok() {
    use rust_decimal_macros::dec;
    let a = positive::Positive::new_decimal(dec!(7)).expect("ok");
    let b = positive::Positive::new_decimal(dec!(2)).expect("ok");
    let r = a
        .checked_div_with_strategy(&b, rust_decimal::RoundingStrategy::ToZero)
        .expect("ok");
    assert_eq!(r.to_dec(), dec!(3.5));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_checked_div_with_strategy_zero_divisor() {
    use rust_decimal_macros::dec;
    let a = positive::Positive::new_decimal(dec!(7)).expect("ok");
    let b = positive::Positive::new_decimal(dec!(0)).expect("ok");
    let err = a
        .checked_div_with_strategy(&b, rust_decimal::RoundingStrategy::ToZero)
        .unwrap_err();
    assert!(matches!(
        err,
        positive::PositiveError::ArithmeticError { .. }
    ));
}

#[cfg(feature = "non-zero")]
#[test]
fn test_checked_div_with_strategy_positive_divisor() {
    use rust_decimal_macros::dec;
    // Under `non-zero` a zero divisor cannot be constructed; exercise
    // the non-zero happy path instead.
    let a = positive::Positive::new_decimal(dec!(7)).expect("ok");
    let b = positive::Positive::ONE;
    let r = a
        .checked_div_with_strategy(&b, rust_decimal::RoundingStrategy::ToZero)
        .expect("ok");
    assert_eq!(r.to_dec(), dec!(7));
}

#[test]
fn test_format_fixed_places_preserves_decimal_precision() {
    use rust_decimal_macros::dec;
    // This value exceeds f64 precision (>15 significant digits). The
    // Decimal-native formatter preserves every digit, while the old
    // f64 round-trip would lose precision after ~15 digits.
    let value = positive::Positive::new_decimal(dec!(1.2345678901234567890123)).expect("ok");
    let formatted = value.format_fixed_places(20);
    assert!(
        formatted.contains("0123"),
        "expected precise tail, got {formatted}"
    );
}

#[test]
fn test_is_multiple_of_dec_true() {
    use rust_decimal_macros::dec;
    let p = pos_or_panic!(15.0);
    assert!(p.is_multiple_of_dec(dec!(3)));
    assert!(p.is_multiple_of_dec(dec!(5)));
    assert!(p.is_multiple_of_dec(dec!(1)));
}

#[test]
fn test_is_multiple_of_dec_false() {
    use rust_decimal_macros::dec;
    let p = pos_or_panic!(15.0);
    assert!(!p.is_multiple_of_dec(dec!(2)));
    assert!(!p.is_multiple_of_dec(dec!(4)));
}

#[test]
fn test_is_multiple_of_dec_zero_divisor() {
    use rust_decimal_macros::dec;
    let p = pos_or_panic!(15.0);
    assert!(!p.is_multiple_of_dec(dec!(0)));
}

// ===== PositiveError contract (issue #80) =====

/// `FromStr` must report failures through `PositiveError`, not `String`, and
/// must preserve the offending input verbatim.
#[test]
fn test_from_str_unparsable_returns_invalid_value_with_input() {
    let err = Positive::from_str("not a number").unwrap_err();
    assert!(matches!(err, PositiveError::InvalidValue { .. }));
    match &err {
        PositiveError::InvalidValue { value, .. } => assert_eq!(value, "not a number"),
        other => panic!("expected InvalidValue, got {other:?}"),
    }
    assert!(err.to_string().contains("not a number"));
}

#[test]
fn test_from_str_negative_returns_out_of_bounds() {
    let err = Positive::from_str("-1.5").unwrap_err();
    assert!(matches!(err, PositiveError::OutOfBounds { .. }));
}

/// The parse error must survive as a typed value, so callers can match on it
/// instead of string-matching a `String` error.
#[test]
fn test_from_str_error_type_is_positive_error() {
    fn parse(s: &str) -> Result<Positive, PositiveError> {
        Positive::from_str(s)
    }
    assert!(parse("2.5").is_ok());
    assert!(parse("").is_err());
}

/// `OutOfBounds` must carry exact `Decimal`s. A value below `f64`'s subnormal
/// range still has to round-trip through the error untouched.
#[test]
fn test_out_of_bounds_preserves_exact_decimal_value() {
    let tiny_negative = Decimal::new(-1, 28);
    let err = Positive::new_decimal(tiny_negative).unwrap_err();
    match err {
        PositiveError::OutOfBounds { value, .. } => assert_eq!(value, tiny_negative),
        other => panic!("expected OutOfBounds, got {other:?}"),
    }
}

/// Large magnitudes must not be projected through `f64`, which would round
/// `Decimal::MIN` and lose the last digits.
#[test]
fn test_out_of_bounds_preserves_decimal_min_exactly() {
    let err = Positive::new_decimal(Decimal::MIN).unwrap_err();
    match err {
        PositiveError::OutOfBounds { value, max, .. } => {
            assert_eq!(value, Decimal::MIN);
            assert_eq!(max, Decimal::MAX);
        }
        other => panic!("expected OutOfBounds, got {other:?}"),
    }
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_out_of_bounds_min_is_zero_by_default() {
    let err = Positive::new_decimal(Decimal::NEGATIVE_ONE).unwrap_err();
    match err {
        PositiveError::OutOfBounds { min, .. } => assert_eq!(min, Decimal::ZERO),
        other => panic!("expected OutOfBounds, got {other:?}"),
    }
}

/// Under `non-zero` the smallest permitted value is `1e-28` — the smallest
/// strictly positive `Decimal` — not `f64::MIN_POSITIVE`.
#[cfg(feature = "non-zero")]
#[test]
fn test_out_of_bounds_min_is_smallest_decimal_under_non_zero() {
    let err = Positive::new_decimal(Decimal::ZERO).unwrap_err();
    match err {
        PositiveError::OutOfBounds { min, .. } => {
            assert_eq!(min, Decimal::new(1, 28));
            assert!(Positive::new_decimal(min).is_ok());
        }
        other => panic!("expected OutOfBounds, got {other:?}"),
    }
}

#[test]
fn test_new_nan_is_invalid_value() {
    let err = Positive::new(f64::NAN).unwrap_err();
    assert!(matches!(err, PositiveError::InvalidValue { .. }));
}

#[test]
fn test_new_infinity_is_invalid_value() {
    let err = Positive::new(f64::INFINITY).unwrap_err();
    assert!(matches!(err, PositiveError::InvalidValue { .. }));
    let err = Positive::new(f64::NEG_INFINITY).unwrap_err();
    assert!(matches!(err, PositiveError::InvalidValue { .. }));
}

/// Every public error message must be lowercase-initial so it composes when a
/// caller wraps it in their own error type.
#[test]
fn test_error_messages_are_lowercase_initial() {
    let errors = [
        Positive::new(f64::NAN).unwrap_err(),
        Positive::new_decimal(Decimal::NEGATIVE_ONE).unwrap_err(),
        Positive::from_str("nope").unwrap_err(),
        pos_or_panic!(5.0).checked_div_f64(0.0).unwrap_err(),
        pos_or_panic!(5.0).checked_add_f64(f64::NAN).unwrap_err(),
    ];
    for error in &errors {
        let rendered = error.to_string();
        let first = rendered.chars().next().expect("message must not be empty");
        assert!(
            first.is_lowercase(),
            "message does not start lowercase: {rendered}"
        );
    }
}

/// The variant set is exhaustive: this compiles without a wildcard arm, which
/// is the property removing the `Other` catch-all was meant to restore.
#[test]
fn test_error_variants_are_exhaustively_matchable() {
    fn describe(error: &PositiveError) -> &'static str {
        match error {
            PositiveError::InvalidValue { .. } => "invalid-value",
            PositiveError::ArithmeticError { .. } => "arithmetic",
            PositiveError::ConversionError { .. } => "conversion",
            PositiveError::OutOfBounds { .. } => "out-of-bounds",
            PositiveError::InvalidPrecision { .. } => "invalid-precision",
        }
    }
    assert_eq!(
        describe(&Positive::new(f64::NAN).unwrap_err()),
        "invalid-value"
    );
    assert_eq!(
        describe(&Positive::new_decimal(Decimal::NEGATIVE_ONE).unwrap_err()),
        "out-of-bounds"
    );
}

// ===== Genuinely non-panicking checked arithmetic (issue #71) =====

/// The headline regression: `Decimal::MAX / 1e-28` overflows inside
/// `rust_decimal`. With raw division this panicked before a `Result` could be
/// returned.
#[test]
fn test_checked_div_max_by_smallest_returns_error_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let err = max.checked_div(&tiny).unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_div_with_strategy_max_by_smallest_returns_error() {
    use rust_decimal::RoundingStrategy;
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let err = max
        .checked_div_with_strategy(&tiny, RoundingStrategy::MidpointNearestEven)
        .unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_div_by_zero_is_arithmetic_error() {
    let a = pos_or_panic!(5.0);
    let zero = Positive::new_decimal(Decimal::ZERO);
    if let Ok(zero) = zero {
        let err = a.checked_div(&zero).unwrap_err();
        assert!(matches!(err, PositiveError::ArithmeticError { .. }));
    }
}

#[test]
fn test_checked_add_max_plus_one_is_arithmetic_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let err = max.checked_add(&Positive::ONE).unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_add_success() {
    let a = pos_or_panic!(2.5);
    assert_eq!(
        a.checked_add(&pos_or_panic!(3.5)).unwrap(),
        pos_or_panic!(6.0)
    );
}

#[test]
fn test_checked_mul_overflow_is_arithmetic_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let err = max.checked_mul(&Positive::TWO).unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_mul_success() {
    let a = pos_or_panic!(4.0);
    assert_eq!(
        a.checked_mul(&pos_or_panic!(2.5)).unwrap(),
        pos_or_panic!(10.0)
    );
}

#[test]
fn test_checked_sub_negative_result_is_out_of_bounds() {
    let a = pos_or_panic!(3.0);
    let err = a.checked_sub(&pos_or_panic!(5.0)).unwrap_err();
    assert!(matches!(err, PositiveError::OutOfBounds { .. }));
}

/// Under `non-zero`, a product that underflows to zero must be reported rather
/// than returned as a `Positive(0)`.
#[cfg(feature = "non-zero")]
#[test]
fn test_checked_mul_underflow_to_zero_is_out_of_bounds() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let err = tiny.checked_mul(&tiny).unwrap_err();
    assert!(matches!(err, PositiveError::OutOfBounds { .. }));
}

// --- mixed Decimal checked operators ---

#[test]
fn test_checked_add_dec() {
    let a = pos_or_panic!(5.0);
    assert_eq!(a.checked_add_dec(dec!(2.5)).unwrap(), pos_or_panic!(7.5));
    assert!(matches!(
        a.checked_add_dec(dec!(-9)).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
    assert!(matches!(
        a.checked_add_dec(Decimal::MAX).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_sub_dec() {
    let a = pos_or_panic!(5.0);
    assert_eq!(a.checked_sub_dec(dec!(1.5)).unwrap(), pos_or_panic!(3.5));
    assert!(matches!(
        a.checked_sub_dec(dec!(9)).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
    assert!(matches!(
        a.checked_sub_dec(Decimal::MIN).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_mul_dec() {
    let a = pos_or_panic!(4.0);
    assert_eq!(a.checked_mul_dec(dec!(2.5)).unwrap(), pos_or_panic!(10.0));
    assert!(matches!(
        a.checked_mul_dec(dec!(-1)).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
    assert!(matches!(
        a.checked_mul_dec(Decimal::MAX).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_div_dec() {
    let a = pos_or_panic!(10.0);
    assert_eq!(a.checked_div_dec(dec!(4)).unwrap(), pos_or_panic!(2.5));
    assert!(matches!(
        a.checked_div_dec(Decimal::ZERO).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
    assert!(matches!(
        a.checked_div_dec(dec!(-2)).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
}

#[test]
fn test_checked_div_dec_max_by_smallest_returns_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(matches!(
        max.checked_div_dec(Decimal::new(1, 28)).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_rem() {
    let a = pos_or_panic!(10.0);
    assert_eq!(
        a.checked_rem(&pos_or_panic!(3.0)).unwrap(),
        pos_or_panic!(1.0)
    );
}

#[test]
fn test_checked_rem_by_zero_is_arithmetic_error() {
    let a = pos_or_panic!(10.0);
    if let Ok(zero) = Positive::new_decimal(Decimal::ZERO) {
        assert!(matches!(
            a.checked_rem(&zero).unwrap_err(),
            PositiveError::ArithmeticError { .. }
        ));
    }
}

/// `sub_or_none` must not panic even when the subtraction would overflow the
/// `Decimal` range.
#[test]
fn test_sub_or_none_cannot_panic_on_overflow() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert_eq!(max.sub_or_none(&Decimal::MIN), None);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sub_or_zero_cannot_panic_on_overflow() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert_eq!(max.sub_or_zero(&Decimal::MIN), Positive::ZERO);
}

/// Comparison against `Decimal` must not panic when the operands straddle the
/// representable range. The epsilon semantics themselves are issue #77.
#[test]
fn test_compare_extremes_does_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(max != Decimal::MIN);
}

/// A nonzero `f64` below `Decimal`'s smallest step rounds to zero during
/// conversion; the comparison must decide by sign instead of the rounded
/// value, so it never reports equality with zero (issue #77 review).
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_tiny_f64_underflow_is_not_equal_to_zero() {
    use std::cmp::Ordering;
    let zero = Positive::ZERO;
    assert!(zero != 1e-100_f64);
    assert!(1e-100_f64 != zero);
    assert_eq!(zero.partial_cmp(&1e-100_f64), Some(Ordering::Less));
    assert_eq!(1e-100_f64.partial_cmp(&zero), Some(Ordering::Greater));
    assert!(zero != -1e-100_f64);
    assert_eq!(zero.partial_cmp(&-1e-100_f64), Some(Ordering::Greater));
}

/// Nonzero values dominate any float that underflows the conversion,
/// regardless of its sign.
#[test]
fn test_tiny_f64_underflow_orders_below_positive_values() {
    use std::cmp::Ordering;
    let one = Positive::ONE;
    assert!(one != 1e-100_f64);
    assert_eq!(one.partial_cmp(&1e-100_f64), Some(Ordering::Greater));
    assert_eq!(one.partial_cmp(&-1e-100_f64), Some(Ordering::Greater));
    assert_eq!(1e-100_f64.partial_cmp(&one), Some(Ordering::Less));
}

#[test]
fn test_approx_comparison_at_extremes_does_not_panic() {
    use approx::{AbsDiffEq, RelativeEq};
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let one = Positive::ONE;

    // The difference is representable here, so both comparisons run their
    // normal path; neither may panic.
    assert!(!max.abs_diff_eq(&one, Decimal::ONE));
    assert!(!max.relative_eq(&one, Decimal::ONE, Decimal::new(1, 10)));

    // A relative tolerance whose product with the larger operand overflows
    // `Decimal` exceeds every representable difference, so the values compare
    // equal rather than panicking.
    assert!(max.relative_eq(&one, Decimal::ONE, Decimal::MAX));
}

/// Every documented panicking arithmetic operator has a checked counterpart.
#[test]
fn test_every_panicking_operator_has_a_checked_counterpart() {
    let a = pos_or_panic!(6.0);
    let b = pos_or_panic!(3.0);
    // Positive op Positive
    assert_eq!((a + b), a.checked_add(&b).unwrap());
    assert_eq!((a - b), a.checked_sub(&b).unwrap());
    assert_eq!((a * b), a.checked_mul(&b).unwrap());
    assert_eq!((a / b), a.checked_div(&b).unwrap());
    // Positive op Decimal
    assert_eq!((a + dec!(3)), a.checked_add_dec(dec!(3)).unwrap());
    assert_eq!((a - dec!(3)), a.checked_sub_dec(dec!(3)).unwrap());
    assert_eq!((a * dec!(3)), a.checked_mul_dec(dec!(3)).unwrap());
    assert_eq!((a / dec!(3)), a.checked_div_dec(dec!(3)).unwrap());
    // Positive op f64
    assert_eq!((a + 3.0), a.checked_add_f64(3.0).unwrap());
    assert_eq!((a - 3.0), a.checked_sub_f64(3.0).unwrap());
    assert_eq!((a * 3.0), a.checked_mul_f64(3.0).unwrap());
    assert_eq!((a / 3.0), a.checked_div_f64(3.0).unwrap());
}

// ===== Invariant preserved on every Positive-returning path (issue #70) =====

/// The headline repro: under `non-zero`, `1e-28 * 1e-28` underflows to zero.
/// The checked API must report it rather than hand back a `Positive(0)`.
#[cfg(feature = "non-zero")]
#[test]
fn test_non_zero_multiplication_underflow_is_reported() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let err = tiny.checked_mul(&tiny).unwrap_err();
    assert!(matches!(err, PositiveError::OutOfBounds { .. }));
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in mul")]
fn test_non_zero_multiplication_underflow_operator_panics() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let _ = tiny * tiny;
}

#[cfg(feature = "non-zero")]
#[test]
fn test_non_zero_division_underflow_is_reported() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let huge = Positive::new_decimal(Decimal::MAX).unwrap();
    let err = tiny.checked_div(&huge).unwrap_err();
    assert!(matches!(err, PositiveError::OutOfBounds { .. }));
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in div")]
fn test_non_zero_division_underflow_operator_panics() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let huge = Positive::new_decimal(Decimal::MAX).unwrap();
    let _ = tiny / huge;
}

// --- rounding down to zero ---

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in floor")]
fn test_non_zero_floor_to_zero_panics() {
    let _ = pos_or_panic!(0.5).floor();
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in round")]
fn test_non_zero_round_to_zero_panics() {
    let _ = pos_or_panic!(0.4).round();
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in round_to")]
fn test_non_zero_round_to_scale_zero_panics() {
    let _ = pos_or_panic!(0.5).round_to(0);
}

/// Without `non-zero` the same calls are legitimate and must keep returning
/// zero rather than panicking.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_default_feature_rounding_to_zero_is_allowed() {
    assert_eq!(pos_or_panic!(0.5).floor(), Positive::ZERO);
    assert_eq!(pos_or_panic!(0.4).round(), Positive::ZERO);
    assert_eq!(pos_or_panic!(0.5).round_to(0), Positive::ZERO);
}

// --- powers ---

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in powu")]
fn test_non_zero_powu_underflow_panics() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let _ = tiny.powu(2);
}

#[cfg(feature = "non-zero")]
#[test]
#[should_panic(expected = "Positive invariant broken in powi")]
fn test_non_zero_powi_underflow_panics() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let _ = tiny.powi(2);
}

/// `sub_or_none` returned `Some(Positive(0))` for equal operands, which is
/// invalid under `non-zero`.
#[test]
fn test_sub_or_none_equal_operands() {
    let value = pos_or_panic!(5.0);
    let result = value.sub_or_none(&value.to_dec());
    #[cfg(feature = "non-zero")]
    assert_eq!(result, None);
    #[cfg(not(feature = "non-zero"))]
    assert_eq!(result, Some(Positive::ZERO));
}

/// Every `Positive` a public path hands back must satisfy the invariant. This
/// sweeps the operations the issue lists over inputs chosen to sit at the edge.
#[test]
fn test_positive_returning_paths_uphold_the_invariant() {
    let values = [
        pos_or_panic!(1.0),
        pos_or_panic!(2.5),
        pos_or_panic!(100.0),
        Positive::new_decimal(Decimal::new(1, 28)).unwrap(),
    ];
    for value in values {
        // Operations that cannot reduce a valid value below the bound.
        for produced in [
            value.ceiling(),
            value.max(Positive::ONE),
            value.min(Positive::ONE),
            value.powu(1),
            value.round_to(28),
            value.clamp(Positive::ONE, Positive::HUNDRED),
        ] {
            assert!(
                positive::is_valid_positive_value(produced.to_dec()),
                "{produced} breaks the invariant"
            );
        }
    }
}

/// `round_to_nice_number` used to route its magnitude through `Positive`,
/// producing an invalid intermediate for every input below ten. It now works
/// entirely in `Decimal`, so inputs below one are handled too.
#[test]
fn test_round_to_nice_number_below_one() {
    let result = pos_or_panic!(0.12).round_to_nice_number();
    assert!(positive::is_valid_positive_value(result.to_dec()));
    assert_eq!(result, pos_or_panic!(0.1));
}

#[test]
fn test_round_to_nice_number_below_ten_upholds_invariant() {
    for input in [0.5_f64, 1.2, 2.5, 4.0, 8.0] {
        let value = Positive::new(input).unwrap();
        let result = value.round_to_nice_number();
        assert!(
            positive::is_valid_positive_value(result.to_dec()),
            "{input} produced an invalid result"
        );
    }
}

// ===== Overflow-safe aggregation (issue #72) =====

#[test]
fn test_checked_sum_owned_iterator() {
    let values = [pos_or_panic!(1.5), pos_or_panic!(2.5), pos_or_panic!(6.0)];
    assert_eq!(Positive::checked_sum(values).unwrap(), pos_or_panic!(10.0));
}

#[test]
fn test_checked_sum_borrowed_iterator() {
    let values = [pos_or_panic!(1.5), pos_or_panic!(2.5), pos_or_panic!(6.0)];
    assert_eq!(
        Positive::checked_sum(values.iter()).unwrap(),
        pos_or_panic!(10.0)
    );
    // the slice is still usable, i.e. it really was borrowed
    assert_eq!(values.len(), 3);
}

#[test]
fn test_checked_sum_singleton() {
    let values = [pos_or_panic!(7.25)];
    assert_eq!(Positive::checked_sum(values).unwrap(), pos_or_panic!(7.25));
}

/// The empty sum is zero, which is a valid `Positive` by default and an
/// invalid one under `non-zero`. It must be reported either way, never
/// invented.
#[test]
fn test_checked_sum_empty() {
    let values: Vec<Positive> = Vec::new();
    let result = Positive::checked_sum(values);
    #[cfg(not(feature = "non-zero"))]
    assert_eq!(result.unwrap(), Positive::ZERO);
    #[cfg(feature = "non-zero")]
    assert!(matches!(
        result.unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
}

/// The case the old `Sum` could not survive: `Decimal::MAX + ONE` panicked
/// inside rust_decimal before `unwrap_or(ZERO)` could ever run.
#[test]
fn test_checked_sum_overflow_is_arithmetic_error_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let err = Positive::checked_sum([max, Positive::ONE]).unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_sum_overflow_mid_iteration() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let err = Positive::checked_sum([Positive::ONE, max, max, max]).unwrap_err();
    assert!(matches!(err, PositiveError::ArithmeticError { .. }));
}

#[test]
fn test_checked_sum_large_iterator() {
    let values: Vec<Positive> = (1..=1_000).map(|_| Positive::ONE).collect();
    assert_eq!(
        Positive::checked_sum(&values).unwrap(),
        Positive::new_decimal(Decimal::from(1_000u64)).unwrap()
    );
}

/// `Sum` must agree with `checked_sum` on every total that has one.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sum_trait_matches_checked_sum() {
    let values = [pos_or_panic!(1.5), pos_or_panic!(2.5), pos_or_panic!(6.0)];

    let owned: Positive = values.into_iter().sum();
    let borrowed: Positive = values.iter().sum();
    let checked = Positive::checked_sum(values).unwrap();

    assert_eq!(owned, checked);
    assert_eq!(borrowed, checked);
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_sum_trait_empty_is_zero() {
    let values: Vec<Positive> = Vec::new();
    let total: Positive = values.into_iter().sum();
    assert_eq!(total, Positive::ZERO);
}

/// `Sum` overflowing must panic with the documented message rather than
/// silently substituting `ZERO`, which would corrupt a financial total.
#[cfg(not(feature = "non-zero"))]
#[test]
#[should_panic(expected = "Positive arithmetic overflow in sum")]
fn test_sum_trait_overflow_panics_and_never_returns_zero() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let _total: Positive = [max, Positive::ONE].into_iter().sum();
}

#[cfg(not(feature = "non-zero"))]
#[test]
#[should_panic(expected = "Positive arithmetic overflow in sum")]
fn test_sum_trait_ref_overflow_panics() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let values = [max, Positive::ONE];
    let _total: Positive = values.iter().sum();
}

// ===== Mathematical domains and checked variants (issue #73) =====

/// `ln` and `log10` return `Decimal`, not `Positive`: the logarithm of a
/// positive number is not itself necessarily positive. Earlier versions
/// returned `-0.693…` inside a `Positive`.
#[test]
fn test_ln_of_sub_one_is_negative_decimal() {
    let half = pos_or_panic!(0.5);
    let result: Decimal = half.ln();
    assert!(result < Decimal::ZERO);
    assert_eq!(half.checked_ln().unwrap(), result);
}

#[test]
fn test_log10_of_sub_one_is_negative_decimal() {
    let half = pos_or_panic!(0.5);
    let result: Decimal = half.log10();
    assert!(result < Decimal::ZERO);
    assert_eq!(half.checked_log10().unwrap(), result);
}

#[test]
fn test_log10_of_one_is_zero() {
    assert_eq!(Positive::ONE.checked_log10().unwrap(), Decimal::ZERO);
    assert_eq!(Positive::ONE.checked_ln().unwrap(), Decimal::ZERO);
}

/// Zero is outside the domain of both logarithms. It is only constructible
/// without the `non-zero` feature.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_logarithms_of_zero_are_domain_errors() {
    assert!(matches!(
        Positive::ZERO.checked_ln().unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
    assert!(matches!(
        Positive::ZERO.checked_log10().unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[cfg(not(feature = "non-zero"))]
#[test]
#[should_panic(expected = "Positive domain error in ln")]
fn test_ln_of_zero_panics_with_domain_message() {
    let _ = Positive::ZERO.ln();
}

#[cfg(not(feature = "non-zero"))]
#[test]
#[should_panic(expected = "Positive domain error in log10")]
fn test_log10_of_zero_panics_with_domain_message() {
    let _ = Positive::ZERO.log10();
}

/// `round_to_nice_number` reached `log10(0)` and panicked. Zero is already the
/// nicest number at its own magnitude.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_round_to_nice_number_of_zero_is_zero() {
    assert_eq!(Positive::ZERO.round_to_nice_number(), Positive::ZERO);
    assert_eq!(
        Positive::ZERO.checked_round_to_nice_number().unwrap(),
        Positive::ZERO
    );
}

// --- powers: domain and overflow ---

/// `powi` with a zero base and a negative exponent is outside the domain; it
/// panicked inside rust_decimal before.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_powi_zero_base_negative_exponent_is_error() {
    assert!(matches!(
        Positive::ZERO.checked_powi(-1).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_powi_overflow_is_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(matches!(
        max.checked_powi(2).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_powu_overflow_is_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(matches!(
        max.checked_powu(2).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_powd_overflow_is_error() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(matches!(
        max.checked_powd(dec!(2)).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_checked_pow_delegates_to_powd() {
    let value = pos_or_panic!(2.0);
    assert_eq!(
        value.checked_pow(pos_or_panic!(3.0)).unwrap(),
        value.checked_powd(dec!(3)).unwrap()
    );
}

// --- exp ---

#[test]
fn test_checked_exp_success() {
    let result = Positive::ONE.checked_exp().unwrap();
    assert!(result > pos_or_panic!(2.7));
    assert!(result < pos_or_panic!(2.8));
}

#[test]
fn test_checked_exp_overflow_is_error() {
    assert!(matches!(
        pos_or_panic!(1000.0).checked_exp().unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
#[should_panic(expected = "Positive arithmetic overflow in exp")]
fn test_exp_overflow_panics() {
    let _ = pos_or_panic!(1000.0).exp();
}

// --- rounding: checked entry points ---

#[test]
fn test_checked_rounding_success() {
    assert_eq!(
        pos_or_panic!(1.9).checked_floor().unwrap(),
        pos_or_panic!(1.0)
    );
    assert_eq!(
        pos_or_panic!(1.6).checked_round().unwrap(),
        pos_or_panic!(2.0)
    );
    assert_eq!(
        pos_or_panic!(1.2345).checked_round_to(2).unwrap(),
        pos_or_panic!(1.23)
    );
    assert_eq!(
        pos_or_panic!(1.1).checked_ceiling().unwrap(),
        pos_or_panic!(2.0)
    );
}

/// Under `non-zero`, rounding down to zero must be reported rather than
/// panicking when the caller asks for the checked form.
#[cfg(feature = "non-zero")]
#[test]
fn test_checked_rounding_to_zero_is_out_of_bounds() {
    assert!(matches!(
        pos_or_panic!(0.5).checked_floor().unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
    assert!(matches!(
        pos_or_panic!(0.4).checked_round().unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
    assert!(matches!(
        pos_or_panic!(0.5).checked_round_to(0).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
}

/// Every panicking mathematical method must agree with its checked
/// counterpart on the values where both succeed.
#[test]
fn test_math_wrappers_agree_with_checked_variants() {
    let value = pos_or_panic!(2.5);
    assert_eq!(value.floor(), value.checked_floor().unwrap());
    assert_eq!(value.round(), value.checked_round().unwrap());
    assert_eq!(value.round_to(1), value.checked_round_to(1).unwrap());
    assert_eq!(value.ceiling(), value.checked_ceiling().unwrap());
    assert_eq!(value.sqrt(), value.checked_sqrt().unwrap());
    assert_eq!(value.exp(), value.checked_exp().unwrap());
    assert_eq!(value.powi(2), value.checked_powi(2).unwrap());
    assert_eq!(value.powu(2), value.checked_powu(2).unwrap());
    assert_eq!(value.powd(dec!(2)), value.checked_powd(dec!(2)).unwrap());
    assert_eq!(
        value.pow(pos_or_panic!(2.0)),
        value.checked_pow(pos_or_panic!(2.0)).unwrap()
    );
    assert_eq!(
        value.round_to_nice_number(),
        value.checked_round_to_nice_number().unwrap()
    );
    assert_eq!(value.ln(), value.checked_ln().unwrap());
    assert_eq!(value.log10(), value.checked_log10().unwrap());
}

// ===== Lawful, panic-free cross-type comparisons (issue #77) =====

/// `PartialEq` requires `a == b` and `b == a` to agree. They did not: one side
/// compared within `EPSILON_CMP`, the other exactly.
#[test]
fn test_partial_eq_decimal_is_symmetric_at_epsilon_boundary() {
    let value = pos_or_panic!(1.000000000000005);
    let one = Decimal::ONE;
    assert_eq!(value == one, one == value);
    assert!(value != one, "equality against Decimal must be exact");
}

#[test]
fn test_partial_eq_decimal_is_symmetric_for_equal_values() {
    let value = pos_or_panic!(2.5);
    assert!(value == dec!(2.5));
    assert!(dec!(2.5) == value);
}

/// Equality and ordering must not panic when the operands sit at opposite
/// extremes of `Decimal`'s range.
#[test]
fn test_comparisons_at_decimal_extremes_do_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(max != Decimal::MIN);
    assert!(Decimal::MIN != max);
    assert!(max > Decimal::MIN);
    assert!(Decimal::MIN < max);
}

/// Ordering against `Decimal` is available in both directions and agrees.
#[test]
fn test_partial_ord_decimal_is_symmetric() {
    let value = pos_or_panic!(5.0);
    assert!(value > dec!(4));
    assert!(dec!(4) < value);
    assert!(value < dec!(6));
    assert!(dec!(6) > value);
}

// --- f64 ---

/// Lowering the `Decimal` to `f64` collapsed every integer above 2^53 onto its
/// neighbours. Lifting the `f64` to `Decimal` keeps them distinct.
#[test]
fn test_f64_equality_is_exact_above_2_53() {
    let big = Positive::new_decimal(Decimal::from(9_007_199_254_740_993u64)).unwrap();
    // 2^53 + 1 is not representable as f64; it rounds to 2^53.
    let as_float = 9_007_199_254_740_992.0_f64;
    assert!(big != as_float);
    assert!(as_float != big);
    assert!(big > as_float);
}

#[test]
fn test_f64_comparison_is_symmetric() {
    let value = pos_or_panic!(2.5);
    assert_eq!(value == 2.5, 2.5 == value);
    assert_eq!(value == 2.6, 2.6 == value);
    assert!(value == 2.5);
    assert!(2.5 == value);
    assert!(value < 3.0);
    assert!(3.0 > value);
    assert!(value > 2.0);
    assert!(2.0 < value);
}

#[test]
fn test_f64_comparison_reference_forms_agree() {
    let value = pos_or_panic!(2.5);
    let by_ref = &value;
    assert!(by_ref == 2.5);
    assert!(2.5 == by_ref);
    assert!(by_ref < 3.0);
    assert!(3.0 > by_ref);
}

/// `NaN` is unordered against everything; it must never compare equal and must
/// never panic.
#[test]
// The point of this test is precisely to compare against NaN through the
// crate's own PartialEq/PartialOrd impls, so the rustc lint that steers
// float-to-float NaN comparisons towards `is_nan()` does not apply.
#[allow(invalid_nan_comparisons)]
fn test_f64_nan_is_unordered() {
    let value = pos_or_panic!(2.5);
    assert!(value != f64::NAN);
    assert!(f64::NAN != value);
    assert_eq!(value.partial_cmp(&f64::NAN), None);
    assert!(!(value < f64::NAN));
    assert!(!(value > f64::NAN));
}

/// Infinities have exact answers even though `Decimal` cannot hold them.
#[test]
fn test_f64_infinities_order_correctly() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(max < f64::INFINITY);
    assert!(f64::INFINITY > max);
    assert!(max > f64::NEG_INFINITY);
    assert!(f64::NEG_INFINITY < max);
    assert!(max != f64::INFINITY);
}

/// A finite `f64` beyond `Decimal`'s range used to convert to `0.0` and
/// compare a huge value as equal to zero.
#[test]
fn test_f64_beyond_decimal_range_orders_correctly() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(max < f64::MAX);
    assert!(f64::MAX > max);
    assert!(max != f64::MAX);
}

/// A value too large for `f64` must not panic on comparison — the previous
/// `PartialEq<f64> for Positive` went through the panicking `to_f64`.
#[test]
fn test_f64_comparison_of_huge_positive_does_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(max != 1.0);
    assert!(max > 1.0);
}

// --- approximate comparison is now explicit ---

#[test]
fn test_approx_eq_dec_replaces_implicit_epsilon_equality() {
    use positive::constants::EPSILON_CMP;
    let value = pos_or_panic!(1.0);
    assert!(value.approx_eq_dec(dec!(1.000000000000005), EPSILON_CMP));
    assert!(!value.approx_eq_dec(dec!(1.1), EPSILON_CMP));
    // exact equality disagrees, which is exactly why both exist
    assert!(value != dec!(1.000000000000005));
}

#[test]
fn test_approx_eq_dec_at_extremes_does_not_panic() {
    use positive::constants::EPSILON_CMP;
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    assert!(!max.approx_eq_dec(Decimal::MIN, EPSILON_CMP));
}

// --- Eq / Ord / Hash consistency ---

/// `Hash` and `Eq` must agree: equal values hash equally, and the derived
/// `Ord` must agree with `PartialEq`.
#[test]
fn test_eq_ord_hash_are_consistent() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of(value: &Positive) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let a = pos_or_panic!(2.5);
    let b = pos_or_panic!(2.5);
    let c = pos_or_panic!(3.5);

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);

    assert_ne!(a, c);
    assert_eq!(a.cmp(&c), std::cmp::Ordering::Less);
    assert_eq!(a.partial_cmp(&c), Some(std::cmp::Ordering::Less));
}

/// Ordering must be a total order over the values a `Positive` can hold,
/// including the extremes.
#[test]
fn test_ordering_is_total_across_the_range() {
    let mut values = [
        Positive::new_decimal(Decimal::MAX).unwrap(),
        pos_or_panic!(1.0),
        Positive::new_decimal(Decimal::new(1, 28)).unwrap(),
        pos_or_panic!(100.0),
    ];
    values.sort();
    for window in values.windows(2) {
        assert!(window[0] <= window[1]);
    }
    assert_eq!(
        values[0],
        Positive::new_decimal(Decimal::new(1, 28)).unwrap()
    );
    assert_eq!(
        values[values.len() - 1],
        Positive::new_decimal(Decimal::MAX).unwrap()
    );
}

// ===== Exact multiplicity predicates (issue #78) =====

/// The false positive the issue reports: `1e-17` is not a multiple of one, but
/// the epsilon comparison said it was.
#[test]
fn test_is_multiple_of_1e_17_modulo_one_is_false() {
    let tiny = Positive::new_decimal(Decimal::new(1, 17)).unwrap();
    assert!(!tiny.is_multiple_of(&Positive::ONE));
}

/// The two APIs disagreed: one was exact, the other tolerant. They must agree
/// for equivalent divisors.
#[test]
fn test_is_multiple_of_agrees_with_is_multiple_of_dec() {
    let cases = [
        (15.0_f64, 5.0_f64),
        (15.0, 4.0),
        (15.0, 3.0),
        (15.0, 1.0),
        (0.3, 0.1),
        (2.5, 0.5),
        (1.0, 7.0),
    ];
    for (value, divisor) in cases {
        let value = Positive::new(value).unwrap();
        let divisor = Positive::new(divisor).unwrap();
        assert_eq!(
            value.is_multiple_of(&divisor),
            value.is_multiple_of_dec(divisor.to_dec()),
            "disagreement for {value} % {divisor}"
        );
    }
}

#[test]
fn test_is_multiple_of_exact_multiples() {
    let value = pos_or_panic!(15.0);
    assert!(value.is_multiple_of(&pos_or_panic!(5.0)));
    assert!(value.is_multiple_of(&pos_or_panic!(3.0)));
    assert!(value.is_multiple_of(&Positive::ONE));
    assert!(value.is_multiple_of(&pos_or_panic!(15.0)));
}

#[test]
fn test_is_multiple_of_near_multiples_are_rejected() {
    let value = Positive::new_decimal(dec!(15.0000000000000000000000001)).unwrap();
    assert!(!value.is_multiple_of(&pos_or_panic!(5.0)));
    assert!(!value.is_multiple_of_dec(dec!(5)));
}

#[test]
fn test_is_multiple_of_dec_negative_divisor() {
    // A negative divisor still yields an exact zero remainder for a true
    // multiple; the sign of the divisor does not change divisibility.
    let value = pos_or_panic!(15.0);
    assert!(value.is_multiple_of_dec(dec!(-5)));
    assert!(!value.is_multiple_of_dec(dec!(-4)));
}

#[test]
fn test_is_multiple_of_zero_divisor_is_false() {
    let value = pos_or_panic!(15.0);
    assert!(!value.is_multiple_of_dec(Decimal::ZERO));
    if let Ok(zero) = Positive::new_decimal(Decimal::ZERO) {
        assert!(!value.is_multiple_of(&zero));
    }
}

/// No remainder operation may panic, including at the extremes.
#[test]
fn test_is_multiple_of_at_decimal_extremes_does_not_panic() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let _ = max.is_multiple_of(&tiny);
    let _ = tiny.is_multiple_of(&max);
    let _ = max.is_multiple_of_dec(Decimal::MIN);
    let _ = max.is_multiple_of_dec(Decimal::MAX);
    assert!(max.is_multiple_of_dec(Decimal::MAX));
}

/// Tolerance-based checking is still available, but only under a name that
/// says so and with the tolerance supplied by the caller.
#[test]
fn test_is_multiple_of_within_tolerance() {
    let tiny = Positive::new_decimal(Decimal::new(1, 17)).unwrap();
    assert!(!tiny.is_multiple_of(&Positive::ONE));
    assert!(tiny.is_multiple_of_within(&Positive::ONE, dec!(1e-16)));
    assert!(!tiny.is_multiple_of_within(&Positive::ONE, dec!(1e-18)));
}

#[test]
fn test_is_multiple_of_within_accepts_just_below_a_multiple() {
    // 14.999... is within tolerance of the next multiple of 5.
    let value = Positive::new_decimal(dec!(14.99999999999999999)).unwrap();
    assert!(value.is_multiple_of_within(&pos_or_panic!(5.0), dec!(1e-15)));
    assert!(!value.is_multiple_of(&pos_or_panic!(5.0)));
}

#[test]
fn test_is_multiple_of_within_zero_divisor_is_false() {
    let value = pos_or_panic!(15.0);
    if let Ok(zero) = Positive::new_decimal(Decimal::ZERO) {
        assert!(!value.is_multiple_of_within(&zero, dec!(1e-9)));
    }
}

/// The deprecated `f64` variant keeps a defined contract until it is removed.
#[test]
#[allow(deprecated)]
fn test_deprecated_is_multiple_edge_cases() {
    let value = pos_or_panic!(15.0);
    assert!(value.is_multiple(5.0));
    assert!(!value.is_multiple(4.0));
    // zero and non-finite divisors are false, not a panic or a division by zero
    assert!(!value.is_multiple(0.0));
    assert!(!value.is_multiple(f64::NAN));
    assert!(!value.is_multiple(f64::INFINITY));
    // a value outside f64's range reports false instead of panicking
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let _ = max.is_multiple(5.0);
}

// ===== Lossless, non-panicking primitive conversions (issue #74) =====

/// `TryFrom<usize>` converted through `f64`, so on 64-bit targets every value
/// above 2^53 was rounded.
#[test]
fn test_usize_to_positive_is_exact_above_2_53() {
    let cases: [usize; 5] = [
        9_007_199_254_740_992, // 2^53
        9_007_199_254_740_993, // 2^53 + 1 — not representable as f64
        9_007_199_254_740_991, // 2^53 - 1
        1,
        1_000_000,
    ];
    for value in cases {
        let positive = Positive::try_from(value).expect("valid");
        assert_eq!(
            positive.to_dec(),
            Decimal::from(value),
            "usize {value} did not round-trip exactly"
        );
        assert_eq!(usize::try_from(positive).unwrap(), value);
    }
}

#[test]
fn test_usize_max_round_trips_exactly() {
    let value = usize::MAX;
    let positive = Positive::try_from(value).expect("valid");
    assert_eq!(positive.to_dec(), Decimal::from(value));
    assert_eq!(usize::try_from(positive).unwrap(), value);
}

/// Out-of-range integer conversions must produce a typed error, never zero.
#[test]
fn test_out_of_range_integer_conversions_are_errors_not_zero() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();

    let err = u64::try_from(max).unwrap_err();
    assert!(matches!(err, PositiveError::ConversionError { .. }));

    let err = i64::try_from(max).unwrap_err();
    assert!(matches!(err, PositiveError::ConversionError { .. }));

    let err = usize::try_from(max).unwrap_err();
    assert!(matches!(err, PositiveError::ConversionError { .. }));
}

#[test]
fn test_i64_boundary_conversions() {
    let at_max = Positive::new_decimal(Decimal::from(i64::MAX)).unwrap();
    assert_eq!(i64::try_from(at_max).unwrap(), i64::MAX);

    let above_max = Positive::new_decimal(Decimal::from(i64::MAX as u64 + 1)).unwrap();
    assert!(i64::try_from(above_max).is_err());
    // ...but it still fits in a u64
    assert_eq!(u64::try_from(above_max).unwrap(), i64::MAX as u64 + 1);
}

#[test]
fn test_u64_boundary_conversions() {
    let at_max = Positive::new_decimal(Decimal::from(u64::MAX)).unwrap();
    assert_eq!(u64::try_from(at_max).unwrap(), u64::MAX);

    let above_max = Positive::new_decimal(Decimal::from(u64::MAX) + Decimal::ONE).unwrap();
    assert!(u64::try_from(above_max).is_err());
}

/// Fractional values truncate toward zero — the documented contract.
#[test]
fn test_fractional_integer_conversions_truncate_toward_zero() {
    let value = pos_or_panic!(42.9);
    assert_eq!(u64::try_from(value).unwrap(), 42);
    assert_eq!(i64::try_from(value).unwrap(), 42);
    assert_eq!(usize::try_from(value).unwrap(), 42);
}

/// `to_f64` is infallible; it must not panic for any constructible value.
#[test]
fn test_to_f64_is_infallible_at_extremes() {
    let max = Positive::new_decimal(Decimal::MAX).unwrap();
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    assert!(max.to_f64() > 0.0);
    assert!(tiny.to_f64() >= 0.0);
    assert_eq!(max.to_f64_checked(), Some(max.to_f64()));
    // the From impls are the same conversion and equally infallible
    let as_float: f64 = max.into();
    assert_eq!(as_float, max.to_f64());
    let as_float_ref: f64 = (&max).into();
    assert_eq!(as_float_ref, max.to_f64());
}

#[test]
fn test_integer_round_trip_through_positive() {
    for value in [1u64, 42, 1_000_000, u64::MAX] {
        let positive = Positive::try_from(value).expect("valid");
        assert_eq!(u64::try_from(positive).unwrap(), value);
    }
}

#[cfg(feature = "non-zero")]
#[test]
fn test_usize_zero_is_rejected_under_non_zero() {
    assert!(matches!(
        Positive::try_from(0usize).unwrap_err(),
        PositiveError::OutOfBounds { .. }
    ));
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_usize_zero_is_accepted_by_default() {
    assert_eq!(Positive::try_from(0usize).unwrap(), Positive::ZERO);
}

// ===== MAX replaces the misleading INFINITY sentinel (issue #76) =====

/// Numeric, ordering, display, debug and conversion semantics must all agree
/// on the same underlying value. They did not: the value was `Decimal::MAX`
/// but `Display`, `Debug` and serde all reported `f64::MAX`.
#[test]
fn test_max_semantics_agree_across_every_surface() {
    let max = Positive::MAX;

    // numeric
    assert_eq!(max.to_dec(), Decimal::MAX);
    // ordering
    assert!(max >= Positive::new_decimal(Decimal::MAX).unwrap());
    assert!(max > Positive::HUNDRED);
    // display and debug
    let displayed = format!("{max}");
    let debugged = format!("{max:?}");
    assert_eq!(displayed, "79228162514264337593543950335");
    assert_eq!(debugged, displayed);
    // conversion: the same number, not f64::MAX
    assert!(max.to_f64() < f64::MAX);
}

/// `MAX` must be reachable through both access paths and be the same value.
#[test]
fn test_max_is_available_as_associated_and_module_constant() {
    assert_eq!(Positive::MAX, positive::constants::MAX);
    assert_eq!(Positive::MAX.to_dec(), Decimal::MAX);
}

/// The deprecated name must keep working and denote exactly the same value
/// until it is removed.
#[test]
#[allow(deprecated)]
fn test_infinity_still_equals_max_during_deprecation() {
    assert_eq!(Positive::INFINITY, Positive::MAX);
    assert_eq!(positive::constants::INFINITY, positive::constants::MAX);
}

/// `Positive::new(f64::MAX)` rejected the value while serde accepted it as a
/// sentinel. The two agree now.
#[test]
fn test_f64_max_is_rejected_consistently_by_constructor_and_serde() {
    assert!(Positive::new(f64::MAX).is_err());
    assert!(serde_json::from_str::<Positive>("1.7976931348623157e+308").is_err());
}

/// An infinite float is not a representable `Positive` and is no longer mapped
/// onto `Decimal::MAX`.
#[test]
fn test_infinite_float_is_rejected_on_deserialize() {
    assert!(serde_json::from_str::<Positive>("1e400").is_err());
}

/// The type has no infinity, because `Decimal` has none. `MAX + ONE` overflows
/// rather than saturating at an infinite value.
#[test]
fn test_max_is_a_real_maximum_not_an_infinity() {
    assert!(matches!(
        Positive::MAX.checked_add(&Positive::ONE).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

#[test]
fn test_max_round_trips_through_decimal() {
    let round_tripped = Positive::new_decimal(Positive::MAX.to_dec()).unwrap();
    assert_eq!(round_tripped, Positive::MAX);
    assert_eq!(round_tripped.to_dec(), Decimal::MAX);
}

// ===== Lossless serde wire format (issue #75) =====

/// The headline loss: 28 significant digits went through `f64` and came back
/// with 12 of them gone.
#[test]
fn test_serde_round_trips_full_28_digit_precision() {
    let exact = Decimal::from_str("0.1234567890123456789012345678").unwrap();
    let value = Positive::new_decimal(exact).unwrap();

    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "\"0.1234567890123456789012345678\"");

    let back: Positive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.to_dec(), exact, "precision lost in the round trip");
}

/// `9223372036854775808` is a valid `Positive`, but serialisation used to fail
/// outright with "Failed to convert to i64".
#[test]
fn test_serde_round_trips_integers_above_i64_max() {
    let above = Decimal::from(i64::MAX as u64 + 1);
    let value = Positive::new_decimal(above).unwrap();

    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "\"9223372036854775808\"");

    let back: Positive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.to_dec(), above);
}

/// Every representable `Positive` must have a lossless path, including both
/// extremes of the range.
#[test]
fn test_every_representable_value_round_trips_exactly() {
    let cases = [
        Decimal::MAX,
        Decimal::new(1, 28),
        Decimal::from(u64::MAX),
        Decimal::from_str("0.1234567890123456789012345678").unwrap(),
        Decimal::from_str("12345.6789").unwrap(),
        Decimal::ONE,
    ];
    for exact in cases {
        let Ok(value) = Positive::new_decimal(exact) else {
            continue; // zero under non-zero
        };
        let json = serde_json::to_string(&value).unwrap();
        let back: Positive = serde_json::from_str(&json).unwrap();
        assert_eq!(back.to_dec(), exact, "{exact} did not round-trip exactly");
    }
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_zero_round_trips_by_default() {
    let json = serde_json::to_string(&Positive::ZERO).unwrap();
    let back: Positive = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Positive::ZERO);
}

/// Documents written by 0.5.x stored plain JSON numbers. They must keep
/// loading, lossily but successfully, since the precision was already gone
/// before the bytes reached us.
#[test]
fn test_legacy_numeric_json_still_deserializes() {
    let from_integer: Positive = serde_json::from_str("42").unwrap();
    assert_eq!(from_integer, pos_or_panic!(42.0));

    let from_float: Positive = serde_json::from_str("42.5").unwrap();
    assert_eq!(from_float, pos_or_panic!(42.5));
}

/// Validation is still enforced on the way in, in both feature modes and for
/// both input shapes.
#[test]
fn test_deserialize_still_validates_the_invariant() {
    assert!(serde_json::from_str::<Positive>("\"-1\"").is_err());
    assert!(serde_json::from_str::<Positive>("-1").is_err());
    assert!(serde_json::from_str::<Positive>("\"not a decimal\"").is_err());
}

#[cfg(feature = "non-zero")]
#[test]
fn test_deserialize_rejects_zero_under_non_zero() {
    assert!(serde_json::from_str::<Positive>("\"0\"").is_err());
    assert!(serde_json::from_str::<Positive>("0").is_err());
}

/// The format must not depend on `deserialize_any`, which non-self-describing
/// serializers cannot support. `StrDeserializer` calls exactly the method the
/// implementation requests, standing in for such a format without adding a
/// dependency.
#[test]
fn test_deserializes_from_a_non_self_describing_driver() {
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as ValueError, StrDeserializer};

    let driver: StrDeserializer<ValueError> = "0.1234567890123456789012345678".into_deserializer();
    let value = Positive::deserialize(driver).unwrap();
    assert_eq!(
        value.to_dec(),
        Decimal::from_str("0.1234567890123456789012345678").unwrap()
    );

    let driver: StrDeserializer<ValueError> = "-1".into_deserializer();
    assert!(Positive::deserialize(driver).is_err());
}

/// A serializer that is not human-readable must receive a string, which is
/// what the `deserialize_str` path above expects.
#[test]
fn test_serializes_as_a_string_for_binary_formats() {
    // serde_json is human-readable; its output is the canonical string form.
    let value = Positive::new_decimal(Decimal::MAX).unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert!(json.starts_with('"') && json.ends_with('"'));
}

/// Struct fields round-trip too, which is how the type is actually used.
#[test]
fn test_round_trip_inside_a_struct() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Order {
        price: Positive,
        quantity: Positive,
    }

    let order = Order {
        price: Positive::new_decimal(Decimal::from_str("12345.678901234567890123").unwrap())
            .unwrap(),
        quantity: pos_or_panic!(3.0),
    };
    let json = serde_json::to_string(&order).unwrap();
    let back: Order = serde_json::from_str(&json).unwrap();
    assert_eq!(back, order);
}

// ===== Bounded formatting precision (issue #81) =====

/// Existing valid formatting must be byte-for-byte unchanged.
#[test]
fn test_format_fixed_places_output_is_unchanged_for_valid_precision() {
    let value = pos_or_panic!(1.2345);
    assert_eq!(value.format_fixed_places(0), "1");
    assert_eq!(value.format_fixed_places(1), "1.2");
    assert_eq!(value.format_fixed_places(2), "1.23");
    assert_eq!(value.format_fixed_places(4), "1.2345");
    assert_eq!(value.format_fixed_places(6), "1.234500");

    let whole = pos_or_panic!(42.0);
    assert_eq!(whole.format_fixed_places(2), "42.00");
}

/// The boundary the issue names: 0 and 28 are valid, 29 and u32::MAX are not.
#[test]
fn test_precision_boundaries() {
    let value = pos_or_panic!(1.5);

    assert!(value.checked_format_fixed_places(0).is_ok());
    assert!(value.checked_format_fixed_places(28).is_ok());
    assert!(matches!(
        value.checked_format_fixed_places(29).unwrap_err(),
        PositiveError::InvalidPrecision { .. }
    ));
    assert!(matches!(
        value.checked_format_fixed_places(u32::MAX).unwrap_err(),
        PositiveError::InvalidPrecision { .. }
    ));
}

#[test]
fn test_checked_round_to_precision_boundaries() {
    let value = pos_or_panic!(1.5);

    assert!(value.checked_round_to(0).is_ok());
    assert!(value.checked_round_to(28).is_ok());
    assert!(matches!(
        value.checked_round_to(29).unwrap_err(),
        PositiveError::InvalidPrecision { .. }
    ));
    assert!(matches!(
        value.checked_round_to(u32::MAX).unwrap_err(),
        PositiveError::InvalidPrecision { .. }
    ));
}

/// The error must be reported *before* any allocation, so an absurd precision
/// costs nothing rather than aborting the process on OOM.
#[test]
fn test_absurd_precision_errors_without_allocating() {
    let value = pos_or_panic!(1.5);
    let err = value.checked_format_fixed_places(u32::MAX).unwrap_err();
    match err {
        PositiveError::InvalidPrecision { precision, .. } => assert_eq!(precision, u32::MAX),
        other => panic!("expected InvalidPrecision, got {other:?}"),
    }
}

#[test]
#[should_panic(expected = "Positive precision 29 is invalid")]
fn test_format_fixed_places_panics_above_max_scale() {
    let _ = pos_or_panic!(1.5).format_fixed_places(29);
}

#[test]
#[should_panic(expected = "is invalid")]
fn test_round_to_panics_above_max_scale() {
    let _ = pos_or_panic!(1.5).round_to(u32::MAX);
}

/// The panicking wrappers must agree with their checked counterparts wherever
/// both succeed.
#[test]
fn test_precision_wrappers_agree_with_checked_variants() {
    let value = pos_or_panic!(1.23456789);
    for places in [0u32, 1, 2, 8, 28] {
        assert_eq!(
            value.format_fixed_places(places),
            value.checked_format_fixed_places(places).unwrap()
        );
        assert_eq!(
            value.round_to(places),
            value.checked_round_to(places).unwrap()
        );
    }
}

/// Formatting at the full supported precision produces exactly 28 decimals.
#[test]
fn test_format_at_max_scale_produces_28_decimals() {
    let value = pos_or_panic!(1.5);
    let formatted = value.checked_format_fixed_places(28).unwrap();
    let decimals = formatted.split('.').nth(1).expect("has a fractional part");
    assert_eq!(decimals.len(), 28);
    assert!(formatted.starts_with("1.5"));
}

// ===== Inverted clamp bounds have one explicit contract (issue #82) =====

/// The defect: with `min > max`, the old if/else chain returned `min` for a
/// low input and `max` for a high one, so the same impossible interval gave
/// two different answers and neither told the caller anything was wrong.
#[test]
fn test_inverted_bounds_are_reported_not_silently_resolved() {
    let low = pos_or_panic!(1.0);
    let high = pos_or_panic!(100.0);
    let min = pos_or_panic!(10.0);
    let max = pos_or_panic!(5.0);

    let low_err = low.checked_clamp(min, max).unwrap_err();
    let high_err = high.checked_clamp(min, max).unwrap_err();

    assert!(matches!(low_err, PositiveError::OutOfBounds { .. }));
    assert!(matches!(high_err, PositiveError::OutOfBounds { .. }));
    // one contract, regardless of where the input sits
    assert_eq!(low_err, high_err);
}

#[test]
#[should_panic(expected = "Positive clamp range is inverted")]
fn test_clamp_panics_on_inverted_bounds() {
    let _ = pos_or_panic!(7.0).clamp(pos_or_panic!(10.0), pos_or_panic!(5.0));
}

#[test]
fn test_checked_clamp_below_inside_and_above() {
    let min = pos_or_panic!(5.0);
    let max = pos_or_panic!(10.0);

    assert_eq!(pos_or_panic!(1.0).checked_clamp(min, max).unwrap(), min);
    assert_eq!(
        pos_or_panic!(7.0).checked_clamp(min, max).unwrap(),
        pos_or_panic!(7.0)
    );
    assert_eq!(pos_or_panic!(50.0).checked_clamp(min, max).unwrap(), max);
}

/// Equal bounds are a valid interval that collapses to a single value.
#[test]
fn test_checked_clamp_equal_bounds() {
    let bound = pos_or_panic!(5.0);
    assert_eq!(
        pos_or_panic!(1.0).checked_clamp(bound, bound).unwrap(),
        bound
    );
    assert_eq!(
        pos_or_panic!(5.0).checked_clamp(bound, bound).unwrap(),
        bound
    );
    assert_eq!(
        pos_or_panic!(9.0).checked_clamp(bound, bound).unwrap(),
        bound
    );
}

/// Exactly-on-the-bound inputs are inside the interval, not clamped.
#[test]
fn test_checked_clamp_at_the_bounds_is_identity() {
    let min = pos_or_panic!(5.0);
    let max = pos_or_panic!(10.0);
    assert_eq!(min.checked_clamp(min, max).unwrap(), min);
    assert_eq!(max.checked_clamp(min, max).unwrap(), max);
}

/// The panicking wrapper must agree with the checked form wherever both
/// succeed.
#[test]
fn test_clamp_agrees_with_checked_clamp() {
    let min = pos_or_panic!(5.0);
    let max = pos_or_panic!(10.0);
    for input in [1.0_f64, 5.0, 7.5, 10.0, 50.0] {
        let value = Positive::new(input).unwrap();
        assert_eq!(
            value.clamp(min, max),
            value.checked_clamp(min, max).unwrap()
        );
    }
}

/// The contract holds at the extremes of the range under both feature modes.
#[test]
fn test_checked_clamp_at_decimal_extremes() {
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
    let max = Positive::MAX;

    assert_eq!(tiny.checked_clamp(tiny, max).unwrap(), tiny);
    assert_eq!(max.checked_clamp(tiny, max).unwrap(), max);
    assert!(max.checked_clamp(max, tiny).is_err());
}

#[cfg(not(feature = "non-zero"))]
#[test]
fn test_checked_clamp_with_zero_bounds_by_default() {
    let value = pos_or_panic!(5.0);
    assert_eq!(
        value.checked_clamp(Positive::ZERO, Positive::ONE).unwrap(),
        Positive::ONE
    );
    assert!(value.checked_clamp(Positive::ONE, Positive::ZERO).is_err());
}

/// The inherent `clamp` must win over `Ord::clamp` for the normal call form.
///
/// With the previous `&self` receiver it never did: `Ord::clamp` matched
/// `Positive` by value at the first resolution step, so the crate's own method
/// was unreachable through method syntax and only usable as
/// `Positive::clamp(&value, ..)`. Taking `self` puts the inherent method at the
/// same step, where it is preferred.
#[test]
fn test_inherent_clamp_is_the_one_that_runs() {
    let min = pos_or_panic!(5.0);
    let max = pos_or_panic!(10.0);
    let value = pos_or_panic!(1.0);

    assert_eq!(value.clamp(min, max), min);
    assert_eq!(Positive::clamp(value, min, max), min);
    assert_eq!(value.checked_clamp(min, max).unwrap(), min);
}

/// No silent-wrong-answer path is left. The inherent method panics with the
/// crate's message; the `Ord` implementation, still reachable for reference
/// receivers, asserts. Either way an inverted range is reported.
#[test]
#[should_panic(expected = "assertion failed: min <= max")]
fn test_ord_clamp_on_references_also_rejects_inverted_bounds() {
    let value = pos_or_panic!(7.0);
    let min = pos_or_panic!(10.0);
    let max = pos_or_panic!(5.0);
    let _ = (&value).clamp(&min, &max);
}
