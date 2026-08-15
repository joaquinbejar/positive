/// Asserts that two `Positive` values are relatively equal within a given epsilon.
///
/// This macro compares two `Positive` values and checks if their relative difference
/// is within the specified `epsilon`. It handles cases where one or both values
/// are zero, ensuring that the non-zero value is less than or equal to epsilon.
///
/// # Examples
///
/// ```
/// use positive::Positive;
/// use positive::{assert_pos_relative_eq, pos_or_panic};
///
/// let a = pos_or_panic!(1.0);
/// let b = pos_or_panic!(1.0001);
/// let epsilon = pos_or_panic!(0.001);
/// assert_pos_relative_eq!(a, b, epsilon); // Passes
/// ```
///
/// ```should_panic
/// use positive::{assert_pos_relative_eq, pos_or_panic};
///
/// let c = pos_or_panic!(1.0);
/// let d = pos_or_panic!(2.0);
/// let epsilon = pos_or_panic!(0.001);
/// assert_pos_relative_eq!(c, d, epsilon); // Panics
/// ```
///
/// # Panics
///
/// This macro panics if the relative difference between the two values is greater than
/// the specified `epsilon`, or if one value is zero and the other is greater than epsilon.
/// The panic message includes the values being compared, the expected relative difference,
/// and the actual relative difference.
#[macro_export]
macro_rules! assert_pos_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: $crate::Positive = $left;
        let right: $crate::Positive = $right;
        let epsilon: $crate::Positive = $epsilon;
        let diff_f64 = (left.to_f64() - right.to_f64()).abs();

        // If the difference is exactly zero, the values are equal — always passes.
        if diff_f64 == 0.0 {
            // exact match, nothing to check
        } else {
            // `diff_f64` is an `abs()` of a finite difference and the zero
            // case is handled above, so it is strictly positive here. This
            // macro is a test assertion helper; panicking is its contract.
            let abs_diff: $crate::Positive = $crate::Positive::new(diff_f64)
                .expect("abs_diff must be positive"); // scan-banned: allow
            let max_abs = left.max(right);

            if left.is_zero() || right.is_zero() {
                let non_zero_value = if left.is_zero() {
                    right
                } else {
                    left
                };
                assert!(
                    non_zero_value <= epsilon,
                    "assertion failed: `(left == right)` \
                     (left: `{}`, right: `{}`, expected max value: `{}`, actual value: `{}`)",
                    left,
                    right,
                    epsilon,
                    non_zero_value
                );
            } else {
                let relative_diff = abs_diff / max_abs;
                assert!(
                    relative_diff <= epsilon,
                    "assertion failed: `(left ≈ right)` \
                     (left: `{}`, right: `{}`, expected relative diff: `{}`, real relative diff: `{}`)",
                    left,
                    right,
                    epsilon,
                    relative_diff
                );
            }
        }
    }};
}

#[cfg(test)]
mod tests_assert_positivef64_relative_eq {
    #[cfg(not(feature = "non-zero"))]
    use crate::Positive;

    #[test]
    fn test_exact_equality() {
        let a = pos_or_panic!(1.0);
        let b = pos_or_panic!(1.0);
        let epsilon = pos_or_panic!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_close_values() {
        let a = pos_or_panic!(1.0);
        let b = pos_or_panic!(1.0001);
        let epsilon = pos_or_panic!(0.001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[cfg(not(feature = "non-zero"))]
    #[test]
    fn test_zero_values() {
        let a = Positive::ZERO;
        let b = Positive::ZERO;
        let epsilon = pos_or_panic!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[cfg(not(feature = "non-zero"))]
    #[test]
    fn test_zero_and_small_value() {
        let a = Positive::ZERO;
        let b = pos_or_panic!(0.00001);
        let epsilon = pos_or_panic!(0.00001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_values_exceeding_epsilon() {
        let a = pos_or_panic!(1.0);
        let b = pos_or_panic!(1.002);
        let epsilon = pos_or_panic!(0.001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_large_values() {
        let a = pos_or_panic!(1000000.0);
        let b = pos_or_panic!(1000001.0);
        let epsilon = pos_or_panic!(0.000002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_very_small_values() {
        let a = pos_or_panic!(0.0000001);
        let b = pos_or_panic!(0.0000001000001);
        let epsilon = pos_or_panic!(0.000002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_significantly_different_values() {
        let a = pos_or_panic!(1.0);
        let b = pos_or_panic!(2.0);
        let epsilon = pos_or_panic!(0.1);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_equal_within_epsilon() {
        let a = pos_or_panic!(100.0);
        let b = pos_or_panic!(100.1);
        let epsilon = pos_or_panic!(0.002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[cfg(not(feature = "non-zero"))]
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_zero_and_large_value() {
        let a = Positive::ZERO;
        let b = pos_or_panic!(1.0);
        let epsilon = pos_or_panic!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }
}
