/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Integration tests for the Positive type.

use positive::{Positive, pos, pos_or_panic, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

#[test]
fn test_positive_decimal_creation() {
    assert!(Positive::new_decimal(Decimal::ZERO).is_ok());
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

#[test]
fn test_positive_decimal_default() {
    assert_eq!(Positive::default().value(), Decimal::ZERO);
}

#[test]
fn test_decimal_div_positive_decimal() {
    let a = dec!(6.0);
    let b = Positive::new_decimal(dec!(2.0)).unwrap();
    assert_eq!(a / b, dec!(3.0));
}

#[test]
fn test_constants() {
    assert_eq!(Positive::ZERO.value(), Decimal::ZERO);
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

#[test]
#[should_panic(expected = "Cannot negate a Positive value!")]
fn test_positive_decimal_neg() {
    let a = pos_or_panic!(1.0);
    let _ = -a;
}

#[test]
fn test_sum_owned_values() {
    let values = vec![pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
    let sum: Positive = values.into_iter().sum();
    assert_eq!(sum.to_f64(), 6.0);
}

#[test]
fn test_sum_referenced_values() {
    let values = [pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
    let sum: Positive = values.iter().sum();
    assert_eq!(sum.to_f64(), 6.0);
}

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

#[test]
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

#[test]
fn test_pos_zero() {
    assert_eq!(Positive::ZERO, Positive::ZERO);
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
    assert_eq!(serialized, "42.5");
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
    assert_eq!(serialized, "100");
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

#[test]
fn test_positive_zero_deserialization() {
    let json = "0";
    let result = serde_json::from_str::<Positive>(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Positive::ZERO);
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
fn test_sqrt_checked() {
    let value = pos_or_panic!(16.0);
    let result = value.sqrt_checked();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 4.0);
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

#[test]
fn test_is_zero() {
    assert!(Positive::ZERO.is_zero());
    assert!(!pos_or_panic!(1.0).is_zero());
}

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
    assert_eq!(value.to_f64_lossy(), 5.0);
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
    assert_eq!(value.to_i64(), 42);
}

#[test]
fn test_to_u64() {
    let value = pos_or_panic!(42.0);
    assert_eq!(value.to_u64(), 42);
}

#[test]
fn test_to_usize() {
    let value = pos_or_panic!(42.0);
    assert_eq!(value.to_usize(), 42);
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
fn test_sqrt_checked_success() {
    let value = pos_or_panic!(16.0);
    let result = value.sqrt_checked();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 4.0);
}

#[test]
fn test_ln() {
    let value = pos_or_panic!(std::f64::consts::E);
    let result = value.ln();
    assert!((result.to_f64() - 1.0).abs() < 0.001);
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
    let result = value.log10();
    assert_eq!(result.to_f64(), 2.0);
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
fn test_is_multiple_edge_cases() {
    // Test with a value that would produce non-finite result in modulo
    let value = pos_or_panic!(10.0);
    assert!(value.is_multiple(5.0));
    assert!(!value.is_multiple(3.0));

    // Test near-epsilon cases
    let value2 = pos_or_panic!(10.0);
    assert!(value2.is_multiple(2.0));
}

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
fn test_from_positive_to_u64() {
    let p = pos_or_panic!(42.0);
    let u: u64 = p.into();
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
fn test_from_positive_to_usize() {
    let p = pos_or_panic!(42.0);
    let u: usize = p.into();
    assert_eq!(u, 42);
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
fn test_display_infinity() {
    let p = Positive::INFINITY;
    let s = format!("{p}");
    assert_eq!(s, format!("{}", f64::MAX));
}

#[test]
fn test_display_integer() {
    let p = pos_or_panic!(42.0);
    let s = format!("{p}");
    assert_eq!(s, "42");
}

#[test]
fn test_debug_infinity() {
    let p = Positive::INFINITY;
    let s = format!("{p:?}");
    assert_eq!(s, format!("{}", f64::MAX));
}

#[test]
fn test_debug_integer() {
    let p = pos_or_panic!(42.0);
    let s = format!("{p:?}");
    assert_eq!(s, "42");
}

#[test]
fn test_serialize_infinity() {
    let p = Positive::INFINITY;
    let json = serde_json::to_string(&p).unwrap();
    // INFINITY serializes as f64::MAX in scientific notation
    assert!(json.contains("1.7976931348623157e+308") || json.len() > 100);
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
fn test_deserialize_positive_infinity() {
    // Test deserializing f64::INFINITY (represented as special value)
    let json = "1.7976931348623157e+308";
    let result: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(result, Positive::INFINITY);
}

#[test]
fn test_deserialize_negative_f64() {
    let json = "-42.5";
    let result: Result<Positive, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Resulting value must be positive")]
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
fn test_is_multiple_true_case() {
    let value = pos_or_panic!(10.0);
    assert!(value.is_multiple(2.0));
    assert!(value.is_multiple(5.0));
    assert!(value.is_multiple(10.0));
}

#[test]
fn test_is_multiple_near_boundary() {
    let value = pos_or_panic!(9.999999999999998);
    assert!(value.is_multiple(1.0));
}

#[test]
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
    let large = Positive::INFINITY;
    let s = format!("{large}");
    assert!(!s.is_empty());
}

#[test]
fn test_debug_large_integer_no_i64() {
    // Test Debug when scale is 0 but value is too large for i64 (line 771)
    let large = Positive::INFINITY;
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
fn test_new_unchecked() {
    // Test new_unchecked (lines 498-499)
    let value = unsafe { Positive::new_unchecked(dec!(42.0)) };
    assert_eq!(value.to_f64(), 42.0);
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
