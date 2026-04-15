# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - unreleased

This is a development version of 0.5.0 collecting the M2 Phase 1
rule-compliance and performance refactors (see the milestone
`M2 · Phase 1 — Rule compliance + inlining + profile`). The release is
not yet finalised; do not rely on any intermediate state.

### Added

- `[profile.release]` with thin LTO, single codegen-unit, `opt-level = 3`,
  `strip = true`, and `debug = false` (#10).
- `#[repr(transparent)]` on `Positive` (#11).
- Derived `Eq`, `PartialOrd`, `Ord` on `Positive` with the canonical derive
  ordering (#11). Manual impls removed.
- `#[cold] #[inline(never)]` on every `PositiveError` constructor and on
  the `From<&str>` / `From<String>` impls (#13). Keeps error-formatting
  code out of hot call sites.
- `#[inline]` on every small hot-path helper and trait-impl body in
  `Positive` (#14): `value`, `to_dec`, `to_dec_ref`, `to_f64_*`, `is_zero`,
  `round_to`, `ln`, `exp`, `log10`, `ceiling`, `new_unchecked`,
  `from_decimal_const`, every `From`/`Into`/`PartialEq`/`PartialOrd`, and
  every `Add`/`Sub`/`Mul`/`Div`/`AddAssign`/`MulAssign`/`Neg` impl for
  `Positive` (both sides).
- `#[must_use]` on the remaining public constructors and checked
  arithmetic methods that were missing it (#15): `Positive::new`,
  `Positive::new_decimal`, `Positive::checked_sub`, `Positive::checked_div`.

- Crate-private panic helpers `overflow_panic` and `invariant_panic`
  (#18). Both are `#[cold] #[inline(never)]` and provide a single
  canonical panic site for arithmetic overflow and invariant violations,
  which upcoming operator rewrites (#19–#22) will route through instead
  of `.expect()`.

### Changed

- All `Positive`⇄`Positive` operators (`Add`, `Sub`, `Mul`, `Div` for
  both owned and `&` operands, plus `AddAssign`) now route through
  `Decimal::checked_*` and the new panic helpers (#19) instead of raw
  arithmetic or ad-hoc `panic!`. Overflow and invariant violations
  surface via `overflow_panic` / `invariant_panic` with uniform
  messages. Test panic expectations updated accordingly.
- `Positive`⇄`Decimal` operators (`Add`, `Sub`, `Mul`, `Div` for both
  owned and `&Decimal` operands on both sides, plus `AddAssign`,
  `MulAssign`) now also route through `Decimal::checked_*` (#20). For
  `Positive`-returning ops the invariant is re-checked; for
  `Decimal`-returning ops only overflow is guarded.
- `<Op><f64>` operators (`Add`, `Sub`, `Mul`, `Div` between `Positive`
  and `f64`, plus `Div` for `&Positive`) no longer do a
  `Decimal → f64 → operate → f64 → Decimal` round-trip (#21). They now
  lift the `f64` rhs into `Decimal` once via `Decimal::from_f64` and
  stay in `Decimal` through `checked_*`, improving precision and
  avoiding the lossy hop.
- Public checked `f64` arithmetic API on `Positive` (#22): every
  panicking `<Op><f64>` operator now has a non-panicking
  `Result<Positive, PositiveError>` counterpart:
  `Positive::checked_add_f64`, `checked_sub_f64`, `checked_mul_f64`,
  `checked_div_f64`. Required by rule 52 (checked equivalent must exist
  for every panicking operator).
- Explicit `Div` rounding strategy (#23): `DIV_ROUNDING_STRATEGY` const
  (banker's rounding / `MidpointNearestEven`) drives every `Div` impl
  and `Positive::checked_div` / `checked_div_f64` via the crate-private
  `round_div` helper. Callers who need a different strategy can use the
  new `Positive::checked_div_with_strategy`. Rule 54.
- `Neg for Positive` now routes through `invariant_panic("neg")`
  instead of a bespoke `panic!(...)` string (#24). Panic message is now
  `"Positive invariant broken in neg: result would be non-positive"`;
  `#[should_panic]` test updated accordingly.
- `Positive::format_fixed_places` no longer goes through `f64` before
  formatting (#25). It now rounds the underlying `Decimal` directly
  via `round_dp`, preserving precision beyond the ~15 significant
  digits of `f64`.
- Decision recorded for #26 (serde representation): the manual
  `Serialize`/`Deserialize` impls are retained for 0.5.0. Migrating to
  `#[serde(transparent)]` would switch the wire format from JSON
  numbers (`42`, `12.345`, `f64::MAX` for infinity) to JSON strings
  (`"42"`) because `rust_decimal`'s default serde representation is
  string-based without the optional `serde-with-float` / equivalent
  features. Documented in `src/positive.rs`; revisit in a future
  major version if the numeric JSON shape is no longer required.
- Deserialization visitor no longer double-validates the positivity
  invariant (#27). `visit_i64`, `visit_u64`, and `visit_f64` used to
  call `is_valid_positive_value` *and* `Positive::new_decimal`
  (which re-checks the same invariant); now they call only
  `new_decimal`. Error messages for negative/zero inputs now come from
  `PositiveError::OutOfBounds` rather than the bespoke custom strings.
- `Display` and `Debug` for `Positive` now delegate to
  `Decimal::normalize()` instead of allocating an intermediate `String`
  and calling `trim_end_matches('0').trim_end_matches('.')` (#28). Same
  output for every tested case (integer-valued, fractional,
  `Positive::INFINITY`, very large non-`i64` integers).
- `Positive::is_multiple_of_dec(other: Decimal) -> bool` (#29) —
  `Decimal`-native multiplicity check using `Decimal::checked_rem`.
  Replaces the lossy `f64`-based path. `Positive::is_multiple(f64)` is
  now `#[deprecated(since = "0.5.0")]`; existing callers continue to
  work but emit a deprecation warning.
- `Positive::is_multiple_of(&Positive)` now uses `Decimal::checked_rem`
  (#30) so pathological inputs that could previously panic under raw
  `%` now return `false` instead. Observable behaviour for normal
  inputs is unchanged.
- Audited `src/constants.rs` (#31): every `pub const` is built from
  `dec!(...)` literals, `Decimal` associated constants, or
  `Positive::from_decimal_const`. No runtime initialisation,
  allocations, `OnceCell`, or `lazy_static` anywhere. Documented the
  compile-time guarantee at the top of the module.
- Significantly expanded `Positive::new_unchecked` documentation (#32):
  detailed `# Safety` invariant under both feature flags, a preference
  ladder for choosing between `new_decimal` / `new` / the macros /
  `new_unchecked`, and an explicit UB example. The function body is
  unchanged.
- Evaluated niche optimisation for `Option<Positive>` (#33) and
  **deferred**. `Decimal` carries no niche, so `Option<Positive>` pays
  a discriminant byte today. A sibling `PositiveNonZero` type built on
  `NonZeroU128` + scale would recover the niche but nothing in the
  Criterion suites or downstream reports currently justifies the cost.
  Full analysis lives in the local `doc/niche-optimization-proposal.md`
  (not committed). Revisit once benchmarks or a concrete downstream
  complaint demand it.

### Removed

- **BREAKING:** `impl Neg for Positive` has been removed (#34). The
  previous implementation always panicked, so the code
  `let y = -x;` was a trap that surfaced only at runtime. Callers now
  get a compile-time error instead. Migration: the value you want is
  almost certainly a `Decimal`; call `positive.to_dec().neg()` or
  `-positive.to_dec()` explicitly. The corresponding
  `#[should_panic]` test was removed alongside the impl.
- `EPSILON_CMP` constant (= `1e-14`) in `crate::constants` (#17),
  precomputed once so `PartialEq<Decimal> for Positive` and
  `RelativeEq::default_max_relative` no longer multiply `EPSILON` by
  `Decimal::from(100)` on every call.

### Fixed

- `From<Positive> for usize` now routes through `Decimal::to_u64()`
  instead of `Decimal::to_f64() as usize`, preserving precision for
  large integer values (#16). The observable signature is unchanged;
  fractional values still truncate toward zero as before.

### Changed

- **BREAKING:** the inner `Decimal` field of `Positive` is now private (#12).
  Use `Positive::to_dec()` or `Decimal::from(positive)` to read the
  underlying value. Migration for pattern-matching / destructuring is not
  available; use the accessor.

## [0.4.2] - 2026-04-14

### Fixed

- Replace `3.14_f64` literal in `benches/conversion.rs` with `3.25_f64` so
  CI lint passes under clippy 1.94.0 (`approx_constant` is deny-by-default
  and flagged the literal as an approximation of `f64::consts::PI`).

## [0.4.1] - 2026-04-14

### Added

- Benchmark harness based on [Criterion](https://docs.rs/criterion) with three
  bench targets:
  - `benches/arith.rs` — `Positive`/`Positive` and `Positive`/`f64` operators,
    math functions (`sqrt`, `ln`, `exp`, `log10`), `round_to`, `clamp`,
    `checked_sub`/`sub_or_zero`/`saturating_sub`, `checked_div`,
    `is_multiple_of`.
  - `benches/conversion.rs` — `Positive::new`, `TryFrom` conversions,
    `Positive`-to-primitive conversions, and `Positive::from_str`.
  - `benches/format_serde.rs` — `Display`, `Debug`, `format_fixed_places`,
    and `serde` JSON round-trip across representative inputs (including
    `Positive::INFINITY`).
- Frozen performance baseline `v0.4.0` generated via
  `cargo bench -- --save-baseline v0.4.0`. Subsequent performance phases
  compare against this baseline with
  `cargo bench -- --baseline v0.4.0`. The baseline artefacts live under
  `target/criterion/` and are not committed to the repository.
