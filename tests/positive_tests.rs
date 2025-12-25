/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/

//! Integration tests for the Positive type.

use positive::{Positive, pos, spos};
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
    let a = pos!(1.0);
    let b = pos!(2.0);
    let c = pos!(2.0);

    assert!(a < b);
    assert!(b > a);
    assert!(b >= c);
    assert!(b <= c);
}

#[test]
fn test_positive_decimal_add_assign() {
    let mut a = pos!(1.0);
    let b = pos!(2.0);
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
    let a = pos!(1.0);
    let b = pos!(2.0);
    assert_eq!(a.max(b).value(), dec!(2.0));
    assert_eq!(a.min(b).value(), dec!(1.0));
}

#[test]
fn test_positive_decimal_floor() {
    let a = pos!(1.7);
    assert_eq!(a.floor().value(), dec!(1.0));
}

#[test]
#[should_panic(expected = "Cannot negate a Positive value!")]
fn test_positive_decimal_neg() {
    let a = pos!(1.0);
    let _ = -a;
}

#[test]
fn test_sum_owned_values() {
    let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
    let sum: Positive = values.into_iter().sum();
    assert_eq!(sum.to_f64(), 6.0);
}

#[test]
fn test_sum_referenced_values() {
    let values = [pos!(1.0), pos!(2.0), pos!(3.0)];
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
    let a = pos!(5.0);
    let b = pos!(3.0);
    let result = a.checked_sub(&b);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 2.0);
}

#[test]
fn test_checked_sub_failure() {
    let a = pos!(3.0);
    let b = pos!(5.0);
    let result = a.checked_sub(&b);
    assert!(result.is_err());
}

#[test]
fn test_saturating_sub() {
    let a = pos!(5.0);
    let b = pos!(3.0);
    assert_eq!(a.saturating_sub(&b).to_f64(), 2.0);

    let c = pos!(3.0);
    let d = pos!(5.0);
    assert_eq!(c.saturating_sub(&d), Positive::ZERO);
}

#[test]
fn test_checked_div_success() {
    let a = pos!(6.0);
    let b = pos!(2.0);
    let result = a.checked_div(&b);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 3.0);
}

#[test]
fn test_checked_div_by_zero() {
    let a = pos!(6.0);
    let b = Positive::ZERO;
    let result = a.checked_div(&b);
    assert!(result.is_err());
}

#[test]
fn test_pos_positive_values() {
    assert_eq!(pos!(5.0).value(), Decimal::new(5, 0));
    assert_eq!(pos!(1.5).value(), Decimal::new(15, 1));
    assert_eq!(pos!(0.1).value(), Decimal::new(1, 1));
}

#[test]
fn test_pos_zero() {
    assert_eq!(Positive::ZERO, Positive::ZERO);
}

#[test]
fn test_pos_small_decimals() {
    assert_eq!(pos!(0.0001).value(), Decimal::new(1, 4));
    assert_eq!(pos!(0.00001).value(), Decimal::new(1, 5));
    assert_eq!(pos!(0.000001).value(), Decimal::new(1, 6));
}

#[test]
fn test_pos_large_decimals() {
    let val = 0.1234567890123456;
    let expected = Decimal::from_str("0.1234567890123456").unwrap();
    assert_eq!(pos!(val).value(), expected);
}

#[test]
#[should_panic(expected = "OutOfBounds")]
fn test_pos_negative_values() {
    pos!(-1.0);
}

#[test]
fn test_pos_edge_cases() {
    assert_eq!(
        pos!(1e15).value(),
        Decimal::from_str("1000000000000000").unwrap()
    );

    assert_eq!(
        pos!(1e-15).value(),
        Decimal::from_str("0.000000000000001").unwrap()
    );
}

#[test]
fn test_pos_expressions() {
    assert_eq!(pos!(2.0 + 3.0).value(), Decimal::new(5, 0));
    assert_eq!(pos!(1.5 * 2.0).value(), Decimal::new(3, 0));
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
    let value = pos!(42.5);
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, "42.5");
}

#[test]
fn test_positive_deserialization() {
    let json = "42.5";
    let deserialized: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, pos!(42.5));
}

#[test]
fn test_positive_serialization_whole_number() {
    let value = pos!(100.0);
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, "100");
}

#[test]
fn test_positive_deserialization_whole_number() {
    let json = "100";
    let deserialized: Positive = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, pos!(100.0));
}

#[test]
fn test_positive_roundtrip() {
    let original = pos!(123.456);
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
    let value = pos!(10.5);
    assert_eq!(value.format_fixed_places(2), "10.50");

    let value = pos!(10.0);
    assert_eq!(value.format_fixed_places(3), "10.000");

    let value = pos!(10.567);
    assert_eq!(value.format_fixed_places(2), "10.57");

    let value = pos!(0.1);
    assert_eq!(value.format_fixed_places(4), "0.1000");
}

#[test]
fn test_is_multiple() {
    let num = pos!(10.0);
    assert!(num.is_multiple(2.0));
    assert!(!num.is_multiple(3.0));
}

#[test]
fn test_is_multiple_of() {
    let num = pos!(10.0);
    assert!(num.is_multiple_of(&pos!(2.0)));
    assert!(num.is_multiple_of(&pos!(5.0)));
}

#[test]
fn test_clamp() {
    let value = pos!(5.0);
    assert_eq!(value.clamp(pos!(1.0), pos!(10.0)), pos!(5.0));
    assert_eq!(value.clamp(pos!(6.0), pos!(10.0)), pos!(6.0));
    assert_eq!(value.clamp(pos!(1.0), pos!(4.0)), pos!(4.0));
}

#[test]
fn test_sqrt() {
    let value = pos!(16.0);
    assert_eq!(value.sqrt().to_f64(), 4.0);
}

#[test]
fn test_sqrt_checked() {
    let value = pos!(16.0);
    let result = value.sqrt_checked();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_f64(), 4.0);
}

#[test]
fn test_pow() {
    let value = pos!(2.0);
    assert_eq!(value.pow(pos!(3.0)).to_f64(), 8.0);
}

#[test]
fn test_powi() {
    let value = pos!(2.0);
    assert_eq!(value.powi(3).to_f64(), 8.0);
}

#[test]
fn test_powu() {
    let value = pos!(2.0);
    assert_eq!(value.powu(3).to_f64(), 8.0);
}

#[test]
fn test_ceiling() {
    let value = pos!(1.3);
    assert_eq!(value.ceiling().to_f64(), 2.0);
}

#[test]
fn test_round_to() {
    let value = pos!(1.2345);
    assert_eq!(value.round_to(2).to_f64(), 1.23);
}

#[test]
fn test_is_zero() {
    assert!(Positive::ZERO.is_zero());
    assert!(!pos!(1.0).is_zero());
}

#[test]
fn test_sub_or_zero() {
    let a = pos!(5.0);
    assert_eq!(a.sub_or_zero(&dec!(3.0)).to_f64(), 2.0);
    assert_eq!(a.sub_or_zero(&dec!(10.0)), Positive::ZERO);
}

#[test]
fn test_sub_or_none() {
    let a = pos!(5.0);
    assert!(a.sub_or_none(&dec!(3.0)).is_some());
    assert!(a.sub_or_none(&dec!(10.0)).is_none());
}

#[test]
fn test_to_f64_checked() {
    let value = pos!(5.0);
    assert_eq!(value.to_f64_checked(), Some(5.0));
}

#[test]
fn test_to_f64_lossy() {
    let value = pos!(5.0);
    assert_eq!(value.to_f64_lossy(), 5.0);
}

#[test]
fn test_to_i64_checked() {
    let value = pos!(5.0);
    assert_eq!(value.to_i64_checked(), Some(5));
}

#[test]
fn test_to_u64_checked() {
    let value = pos!(5.0);
    assert_eq!(value.to_u64_checked(), Some(5));
}

#[test]
fn test_to_usize_checked() {
    let value = pos!(5.0);
    assert_eq!(value.to_usize_checked(), Some(5));
}
