/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/25
******************************************************************************/

//! Deterministic boundary matrix for the `Positive` public API.
//!
//! # Why this file exists
//!
//! The stored coverage report showed about 92.5% line coverage and **zero**
//! recorded branch coverage, and every test passed. A small boundary harness
//! nevertheless found ten defects, because the existing tests used small,
//! ordinary values: `5.0`, `42.0`, `123.456`. Line coverage counts whether a
//! line ran, not whether it ran with an input that could break it.
//!
//! Rather than add more ad-hoc cases, this file drives every public API over a
//! fixed matrix of values chosen to sit exactly where the implementation
//! changes behaviour: the invariant bound, the limits of `Decimal`, the limits
//! of `f64`'s integer precision, and the destination ranges of every integer
//! conversion.
//!
//! # What it asserts
//!
//! 1. **No non-panicking API unwinds.** Every `checked_*` method is executed
//!    over the full cartesian product of the matrix inside `catch_unwind`. A
//!    panic fails the test — returning `Err` is the contract.
//! 2. **Every `Positive`-returning API upholds the invariant.** Whatever comes
//!    back satisfies `is_valid_positive_value` under both feature modes, or
//!    the call panics with the documented message rather than handing back an
//!    invalid value.
//! 3. **Serde round-trips exactly** for every value in the matrix.
//! 4. **Comparison is lawful** — symmetric equality and consistent ordering
//!    across the matrix, including against `Decimal` and `f64`.
//!
//! # Coverage tooling limitation
//!
//! `cargo tarpaulin`, which this project uses, does not report branch or
//! condition coverage: its default `ptrace`/`llvm` engines instrument lines
//! only, which is why the stored report showed `0` branches rather than a low
//! number. The line figure therefore cannot be read as evidence that boundary
//! behaviour is exercised. This file is the deliberate compensation for that
//! gap: the matrix is explicit in the source, so what is covered can be read
//! and reviewed directly instead of inferred from a coverage percentage.
//!
//! Property-based testing (`proptest` or `quickcheck`) would generalise this
//! further, but `rules/global_rules.md` requires explicit approval before a new
//! dependency is added, so it is deliberately proposed rather than introduced.

use positive::{Positive, PositiveError, is_valid_positive_value};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

/// The canonical boundary vectors, as raw `Decimal`s.
///
/// Each entry is here because some branch in the crate treats it differently
/// from its neighbours.
fn boundary_decimals() -> Vec<(&'static str, Decimal)> {
    vec![
        // The invariant bound itself, valid by default and rejected under
        // `non-zero`.
        ("zero", Decimal::ZERO),
        // Smallest and largest representable magnitudes.
        ("smallest positive (1e-28)", Decimal::new(1, 28)),
        ("largest (Decimal::MAX)", Decimal::MAX),
        // Around one, where floor/round/log10 change sign or collapse to zero.
        ("just below one", dec!(0.9999999999999999999999999999)),
        ("one", Decimal::ONE),
        ("just above one", dec!(1.0000000000000000000000000001)),
        ("one half", dec!(0.5)),
        // f64's exact-integer boundary.
        ("2^53 - 1", Decimal::from(9_007_199_254_740_991u64)),
        ("2^53", Decimal::from(9_007_199_254_740_992u64)),
        ("2^53 + 1", Decimal::from(9_007_199_254_740_993u64)),
        // Integer destination limits.
        ("i64::MAX", Decimal::from(i64::MAX)),
        ("i64::MAX + 1", Decimal::from(i64::MAX as u64 + 1)),
        ("u64::MAX", Decimal::from(u64::MAX)),
        // Full 28-digit fraction.
        (
            "28-digit fraction",
            Decimal::from_str("0.1234567890123456789012345678").expect("valid"),
        ),
        // An ordinary value, so the matrix is not all extremes.
        ("ordinary", dec!(12345.6789)),
    ]
}

/// The boundary vectors that are constructible under the active feature set.
fn boundary_values() -> Vec<(&'static str, Positive)> {
    boundary_decimals()
        .into_iter()
        .filter_map(|(label, value)| Positive::new_decimal(value).ok().map(|p| (label, p)))
        .collect()
}

/// Values that must be rejected by every constructor under every feature set.
fn invalid_decimals() -> Vec<(&'static str, Decimal)> {
    vec![
        ("negative one", Decimal::NEGATIVE_ONE),
        ("Decimal::MIN", Decimal::MIN),
        ("smallest negative", Decimal::new(-1, 28)),
    ]
}

/// Runs `operation` and fails the test if it unwinds instead of returning.
fn must_not_panic<T>(what: &str, operation: impl FnOnce() -> T) -> T {
    // The panic hook is left in place deliberately: if this fires, the
    // backtrace is the most useful part of the failure.
    catch_unwind(AssertUnwindSafe(operation))
        .unwrap_or_else(|_| panic!("{what} panicked; a non-panicking API must return an error"))
}

// ===========================================================================
// 1. No non-panicking API unwinds, for any pair in the matrix
// ===========================================================================

#[test]
fn test_no_checked_arithmetic_panics_over_the_matrix() {
    let values = boundary_values();
    for (lhs_label, lhs) in &values {
        for (rhs_label, rhs) in &values {
            let pair = format!("{lhs_label} op {rhs_label}");

            must_not_panic(&format!("checked_add({pair})"), || lhs.checked_add(rhs)).ok();
            must_not_panic(&format!("checked_sub({pair})"), || lhs.checked_sub(rhs)).ok();
            must_not_panic(&format!("checked_mul({pair})"), || lhs.checked_mul(rhs)).ok();
            must_not_panic(&format!("checked_div({pair})"), || lhs.checked_div(rhs)).ok();
            must_not_panic(&format!("checked_rem({pair})"), || lhs.checked_rem(rhs)).ok();
            must_not_panic(&format!("checked_pow({pair})"), || lhs.checked_pow(*rhs)).ok();
            must_not_panic(&format!("checked_div_with_strategy({pair})"), || {
                lhs.checked_div_with_strategy(rhs, RoundingStrategy::MidpointNearestEven)
            })
            .ok();
            must_not_panic(&format!("checked_clamp({pair})"), || {
                lhs.checked_clamp(*rhs, *lhs)
            })
            .ok();
            must_not_panic(&format!("checked_sum({pair})"), || {
                Positive::checked_sum([*lhs, *rhs])
            })
            .ok();
            must_not_panic(&format!("is_multiple_of({pair})"), || {
                lhs.is_multiple_of(rhs)
            });
            must_not_panic(&format!("sub_or_none({pair})"), || {
                lhs.sub_or_none(&rhs.to_dec())
            });
        }
    }
}

#[test]
fn test_no_checked_mixed_operand_api_panics_over_the_matrix() {
    let values = boundary_values();
    let decimals = boundary_decimals();
    for (lhs_label, lhs) in &values {
        for (rhs_label, rhs) in &decimals {
            let pair = format!("{lhs_label} op {rhs_label}");
            must_not_panic(&format!("checked_add_dec({pair})"), || {
                lhs.checked_add_dec(*rhs)
            })
            .ok();
            must_not_panic(&format!("checked_sub_dec({pair})"), || {
                lhs.checked_sub_dec(*rhs)
            })
            .ok();
            must_not_panic(&format!("checked_mul_dec({pair})"), || {
                lhs.checked_mul_dec(*rhs)
            })
            .ok();
            must_not_panic(&format!("checked_div_dec({pair})"), || {
                lhs.checked_div_dec(*rhs)
            })
            .ok();
            must_not_panic(&format!("is_multiple_of_dec({pair})"), || {
                lhs.is_multiple_of_dec(*rhs)
            });
        }
        // Negative and non-finite operands on the mixed paths.
        for rhs in [Decimal::NEGATIVE_ONE, Decimal::MIN, Decimal::ZERO] {
            must_not_panic("checked_add_dec(negative)", || lhs.checked_add_dec(rhs)).ok();
            must_not_panic("checked_sub_dec(negative)", || lhs.checked_sub_dec(rhs)).ok();
            must_not_panic("checked_mul_dec(negative)", || lhs.checked_mul_dec(rhs)).ok();
            must_not_panic("checked_div_dec(negative)", || lhs.checked_div_dec(rhs)).ok();
        }
        for rhs in [0.0_f64, -1.0, f64::NAN, f64::INFINITY, f64::MAX, f64::MIN] {
            must_not_panic("checked_add_f64", || lhs.checked_add_f64(rhs)).ok();
            must_not_panic("checked_sub_f64", || lhs.checked_sub_f64(rhs)).ok();
            must_not_panic("checked_mul_f64", || lhs.checked_mul_f64(rhs)).ok();
            must_not_panic("checked_div_f64", || lhs.checked_div_f64(rhs)).ok();
        }
    }
}

#[test]
fn test_no_checked_mathematical_api_panics_over_the_matrix() {
    for (label, value) in boundary_values() {
        must_not_panic(&format!("checked_floor({label})"), || value.checked_floor()).ok();
        must_not_panic(&format!("checked_round({label})"), || value.checked_round()).ok();
        must_not_panic(&format!("checked_ceiling({label})"), || {
            value.checked_ceiling()
        })
        .ok();
        must_not_panic(&format!("checked_sqrt({label})"), || value.checked_sqrt()).ok();
        // The deprecated alias must stay panic-free until it is removed.
        #[allow(deprecated)]
        must_not_panic(&format!("sqrt_checked({label})"), || value.sqrt_checked()).ok();
        must_not_panic(&format!("to_f64_checked({label})"), || {
            value.to_f64_checked()
        });
        must_not_panic(&format!("checked_exp({label})"), || value.checked_exp()).ok();
        must_not_panic(&format!("checked_ln({label})"), || value.checked_ln()).ok();
        must_not_panic(&format!("checked_log10({label})"), || value.checked_log10()).ok();
        must_not_panic(&format!("checked_round_to_nice_number({label})"), || {
            value.checked_round_to_nice_number()
        })
        .ok();

        for exponent in [0i64, 1, 2, -1, -2, i64::MAX, i64::MIN] {
            must_not_panic(&format!("checked_powi({label}, {exponent})"), || {
                value.checked_powi(exponent)
            })
            .ok();
        }
        for exponent in [0u64, 1, 2, u64::MAX] {
            must_not_panic(&format!("checked_powu({label}, {exponent})"), || {
                value.checked_powu(exponent)
            })
            .ok();
        }
        for exponent in [
            Decimal::ZERO,
            Decimal::ONE,
            Decimal::NEGATIVE_ONE,
            Decimal::MAX,
        ] {
            must_not_panic(&format!("checked_powd({label}, {exponent})"), || {
                value.checked_powd(exponent)
            })
            .ok();
        }
        for places in [0u32, 1, 28, 29, u32::MAX] {
            must_not_panic(&format!("checked_round_to({label}, {places})"), || {
                value.checked_round_to(places)
            })
            .ok();
            must_not_panic(
                &format!("checked_format_fixed_places({label}, {places})"),
                || value.checked_format_fixed_places(places),
            )
            .ok();
        }
    }
}

/// Zero raised to a negative power is undefined; the checked power entry
/// points must report it as a domain error instead of a silent zero.
#[cfg(not(feature = "non-zero"))]
#[test]
fn test_zero_to_a_negative_power_is_a_domain_error() {
    assert!(matches!(
        Positive::ZERO
            .checked_powd(Decimal::NEGATIVE_ONE)
            .unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
    assert!(matches!(
        Positive::ZERO.checked_powi(-1).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
}

/// Division by zero must be an error on every division entry point, never a
/// panic and never a silent result.
#[test]
fn test_division_by_zero_is_always_an_error() {
    let value = Positive::new_decimal(dec!(5)).expect("valid");

    assert!(matches!(
        value.checked_div_dec(Decimal::ZERO).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
    assert!(matches!(
        value.checked_div_f64(0.0).unwrap_err(),
        PositiveError::ArithmeticError { .. }
    ));
    if let Ok(zero) = Positive::new_decimal(Decimal::ZERO) {
        assert!(matches!(
            value.checked_div(&zero).unwrap_err(),
            PositiveError::ArithmeticError { .. }
        ));
        assert!(matches!(
            value.checked_rem(&zero).unwrap_err(),
            PositiveError::ArithmeticError { .. }
        ));
    }
}

/// Conversions must report, never panic and never silently produce zero.
#[test]
fn test_no_conversion_panics_over_the_matrix() {
    for (label, value) in boundary_values() {
        must_not_panic(&format!("to_f64({label})"), || value.to_f64());
        must_not_panic(&format!("to_i64_checked({label})"), || {
            value.to_i64_checked()
        });
        must_not_panic(&format!("to_u64_checked({label})"), || {
            value.to_u64_checked()
        });
        must_not_panic(&format!("to_usize_checked({label})"), || {
            value.to_usize_checked()
        });
        must_not_panic(&format!("u64::try_from({label})"), || u64::try_from(value)).ok();
        must_not_panic(&format!("i64::try_from({label})"), || i64::try_from(value)).ok();
        must_not_panic(&format!("usize::try_from({label})"), || {
            usize::try_from(value)
        })
        .ok();

        // An out-of-range conversion must be an error, not zero.
        if let Err(error) = u64::try_from(value) {
            assert!(matches!(error, PositiveError::ConversionError { .. }));
        } else if let Ok(converted) = u64::try_from(value) {
            assert_eq!(
                Decimal::from(converted),
                value.to_dec().trunc(),
                "{label} converted to a different number"
            );
        }
    }
}

// ===========================================================================
// 2. Every Positive-returning API upholds the invariant
// ===========================================================================

/// Whatever a `Positive`-returning API hands back must satisfy the invariant
/// for the active feature set — or the call must panic rather than return an
/// invalid value. Both outcomes are acceptable; silently returning something
/// invalid is not.
#[test]
fn test_every_positive_returning_api_upholds_the_invariant() {
    let values = boundary_values();

    for (label, value) in &values {
        let mut produced: Vec<(String, Result<Positive, ()>)> = Vec::with_capacity(32);

        macro_rules! record {
            ($name:expr, $call:expr) => {
                produced.push((
                    format!("{}({})", $name, label),
                    catch_unwind(AssertUnwindSafe(|| $call)).map_err(|_| ()),
                ));
            };
        }

        record!("floor", value.floor());
        record!("round", value.round());
        record!("ceiling", value.ceiling());
        record!("sqrt", value.sqrt());
        record!("round_to(2)", value.round_to(2));
        record!("round_to_nice_number", value.round_to_nice_number());
        record!("powu(1)", value.powu(1));
        record!("powu(2)", value.powu(2));
        record!("powi(2)", value.powi(2));
        record!("min(ONE)", (*value).min(Positive::ONE));
        record!("max(ONE)", (*value).max(Positive::ONE));
        record!(
            "clamp(ONE, MAX)",
            (*value).clamp(Positive::ONE, Positive::MAX)
        );

        for (other_label, other) in &values {
            produced.push((
                format!("{label} + {other_label}"),
                catch_unwind(AssertUnwindSafe(|| *value + *other)).map_err(|_| ()),
            ));
            produced.push((
                format!("{label} * {other_label}"),
                catch_unwind(AssertUnwindSafe(|| *value * *other)).map_err(|_| ()),
            ));
            produced.push((
                format!("{label} / {other_label}"),
                catch_unwind(AssertUnwindSafe(|| *value / *other)).map_err(|_| ()),
            ));
        }

        for (what, outcome) in produced {
            if let Ok(result) = outcome {
                assert!(
                    is_valid_positive_value(result.to_dec()),
                    "{what} returned {result}, which breaks the invariant"
                );
            }
        }
    }
}

/// The same sweep for the checked forms: a successful result always satisfies
/// the invariant, and a failure is always a typed error.
#[test]
fn test_every_checked_api_returns_a_valid_value_or_a_typed_error() {
    let values = boundary_values();
    for (lhs_label, lhs) in &values {
        for (rhs_label, rhs) in &values {
            for (name, outcome) in [
                ("checked_add", lhs.checked_add(rhs)),
                ("checked_sub", lhs.checked_sub(rhs)),
                ("checked_mul", lhs.checked_mul(rhs)),
                ("checked_div", lhs.checked_div(rhs)),
                ("checked_rem", lhs.checked_rem(rhs)),
            ] {
                if let Ok(result) = outcome {
                    assert!(
                        is_valid_positive_value(result.to_dec()),
                        "{name}({lhs_label}, {rhs_label}) returned {result}, which breaks the invariant"
                    );
                }
            }
        }
    }
}

/// No constructor accepts a value that breaks the invariant.
#[test]
fn test_no_constructor_accepts_an_invalid_value() {
    for (label, invalid) in invalid_decimals() {
        assert!(
            Positive::new_decimal(invalid).is_err(),
            "new_decimal accepted {label}"
        );
        assert!(
            Positive::try_from(invalid).is_err(),
            "TryFrom<Decimal> accepted {label}"
        );
        assert!(
            Positive::from_str(&invalid.to_string()).is_err(),
            "FromStr accepted {label}"
        );
    }
    for invalid in [-0.5_f64, -1.0, f64::NAN, f64::NEG_INFINITY, f64::MIN] {
        assert!(Positive::new(invalid).is_err(), "new accepted {invalid}");
    }
}

/// Under `non-zero`, zero must be unreachable through every entry point.
#[cfg(feature = "non-zero")]
#[test]
fn test_zero_is_unreachable_under_non_zero() {
    assert!(Positive::new_decimal(Decimal::ZERO).is_err());
    assert!(Positive::new(0.0).is_err());
    assert!(Positive::from_str("0").is_err());
    assert!(Positive::try_from(0usize).is_err());
    assert!(serde_json::from_str::<Positive>("\"0\"").is_err());

    // ...and no arithmetic path can produce it either.
    let tiny = Positive::new_decimal(Decimal::new(1, 28)).expect("valid");
    assert!(tiny.checked_mul(&tiny).is_err());
    assert!(tiny.checked_sub(&tiny).is_err());
    let huge = Positive::new_decimal(Decimal::MAX).expect("valid");
    assert!(tiny.checked_div(&huge).is_err());
}

// ===========================================================================
// 3. Serde round-trips exactly over the matrix
// ===========================================================================

#[test]
fn test_serde_round_trips_every_boundary_value_exactly() {
    for (label, value) in boundary_values() {
        let json = serde_json::to_string(&value)
            .unwrap_or_else(|error| panic!("{label} failed to serialise: {error}"));
        let back: Positive = serde_json::from_str(&json)
            .unwrap_or_else(|error| panic!("{label} failed to deserialise from {json}: {error}"));
        assert_eq!(
            back.to_dec(),
            value.to_dec(),
            "{label} did not round-trip exactly (json was {json})"
        );
    }
}

#[test]
fn test_serde_rejects_every_invalid_value() {
    for (label, invalid) in invalid_decimals() {
        let json = format!("\"{invalid}\"");
        assert!(
            serde_json::from_str::<Positive>(&json).is_err(),
            "deserialisation accepted {label}"
        );
    }
}

// ===========================================================================
// 4. Comparison algebra over the matrix
// ===========================================================================

#[test]
fn test_equality_is_symmetric_over_the_matrix() {
    let values = boundary_values();
    for (lhs_label, lhs) in &values {
        for (rhs_label, rhs) in &values {
            assert_eq!(
                lhs == rhs,
                rhs == lhs,
                "Positive equality is asymmetric for {lhs_label} / {rhs_label}"
            );

            let rhs_dec = rhs.to_dec();
            assert_eq!(
                *lhs == rhs_dec,
                rhs_dec == *lhs,
                "Positive/Decimal equality is asymmetric for {lhs_label} / {rhs_label}"
            );

            let rhs_float = rhs.to_f64();
            assert_eq!(
                *lhs == rhs_float,
                rhs_float == *lhs,
                "Positive/f64 equality is asymmetric for {lhs_label} / {rhs_label}"
            );
        }
    }
}

/// Raw `f64` boundaries that no representable `Decimal` can produce: finite
/// nonzero floats below `Decimal`'s smallest step underflow the conversion,
/// and must order by sign instead of aliasing to zero.
#[test]
fn test_raw_f64_underflow_boundaries_never_alias_to_zero() {
    use std::cmp::Ordering;

    let one = Positive::ONE;
    for tiny in [1e-100_f64, f64::MIN_POSITIVE, -1e-100_f64] {
        assert!(one != tiny, "ONE compared equal to {tiny}");
        assert_eq!(
            one.partial_cmp(&tiny),
            Some(Ordering::Greater),
            "ONE must exceed {tiny}"
        );
    }

    #[cfg(not(feature = "non-zero"))]
    {
        let zero = Positive::ZERO;
        for tiny in [1e-100_f64, f64::MIN_POSITIVE] {
            assert!(zero != tiny, "ZERO compared equal to {tiny}");
            assert!(tiny != zero, "{tiny} compared equal to ZERO");
            assert_eq!(
                zero.partial_cmp(&tiny),
                Some(Ordering::Less),
                "ZERO must be below {tiny}"
            );
        }
        assert!(zero != -1e-100_f64);
        assert_eq!(zero.partial_cmp(&-1e-100_f64), Some(Ordering::Greater));
    }
}

#[test]
fn test_ordering_is_consistent_with_equality_over_the_matrix() {
    let values = boundary_values();
    for (lhs_label, lhs) in &values {
        for (rhs_label, rhs) in &values {
            let context = format!("{lhs_label} vs {rhs_label}");

            // Ord agrees with PartialOrd and with PartialEq.
            assert_eq!(Some(lhs.cmp(rhs)), lhs.partial_cmp(rhs), "{context}");
            assert_eq!(
                lhs.cmp(rhs) == std::cmp::Ordering::Equal,
                lhs == rhs,
                "{context}"
            );
            // Antisymmetry.
            assert_eq!(lhs.cmp(rhs), rhs.cmp(lhs).reverse(), "{context}");

            // Cross-type ordering agrees with the native one.
            let rhs_dec = rhs.to_dec();
            assert_eq!(
                lhs.partial_cmp(&rhs_dec),
                Some(lhs.cmp(rhs)),
                "Positive/Decimal ordering disagrees for {context}"
            );
        }
    }
}

#[test]
fn test_equal_values_hash_equally_over_the_matrix() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of(value: &Positive) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    for (label, value) in boundary_values() {
        let copy = Positive::new_decimal(value.to_dec()).expect("valid");
        assert_eq!(value, copy, "{label}");
        assert_eq!(hash_of(&value), hash_of(&copy), "{label} hashes unequally");
    }
}

/// Sorting the whole matrix must produce a total order with no comparison
/// panicking, including `Decimal::MAX` against `1e-28`.
#[test]
fn test_sorting_the_matrix_is_a_total_order() {
    let mut values: Vec<Positive> = boundary_values().into_iter().map(|(_, v)| v).collect();
    must_not_panic("sorting the boundary matrix", || values.sort());
    for window in values.windows(2) {
        assert!(window[0] <= window[1]);
    }
}

/// Regression for the defect this matrix found on its first run.
///
/// `format!("{:.28}", Decimal::MAX)` routes through `Decimal`'s own formatter,
/// which writes into a fixed-capacity buffer. A 29-digit value at 28 decimal
/// places needs 58 characters and overflowed it, panicking with
/// `CapacityError` inside the dependency — for a valid value at a valid
/// precision, on an API documented as non-panicking.
#[test]
fn test_formatting_max_at_full_precision_does_not_panic() {
    let max = Positive::MAX;
    let formatted = must_not_panic("checked_format_fixed_places(MAX, 28)", || {
        max.checked_format_fixed_places(28)
    })
    .expect("28 is a valid precision");

    let (integer, fraction) = formatted.split_once('.').expect("has a fractional part");
    assert_eq!(integer, "79228162514264337593543950335");
    assert_eq!(fraction.len(), 28);
    assert!(fraction.chars().all(|c| c == '0'));
}

/// The whole formatting surface, over the whole matrix, at every valid
/// precision.
#[test]
fn test_formatting_never_panics_over_the_matrix() {
    for (label, value) in boundary_values() {
        for places in 0..=28u32 {
            let formatted = must_not_panic(
                &format!("checked_format_fixed_places({label}, {places})"),
                || value.checked_format_fixed_places(places),
            )
            .unwrap_or_else(|error| panic!("{label} at {places} places: {error}"));

            if places == 0 {
                assert!(!formatted.contains('.'), "{label} at 0 places: {formatted}");
            } else {
                let fraction = formatted.split_once('.').expect("has a fractional part").1;
                assert_eq!(
                    fraction.len(),
                    places as usize,
                    "{label} at {places} places produced {formatted}"
                );
            }
        }
    }
}
