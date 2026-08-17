# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2026-08-15

Correctness release. Closes issues #70-#90, which together covered the
crate's arithmetic, invariant, comparison, conversion, serialisation and
documentation contracts. It is **breaking**: see *Migration* below.

The theme is that the type's central guarantee — a `Positive` always
satisfies the positivity invariant — was not actually enforced on every
path, and several APIs advertised as non-panicking panicked. The 0.5.x
test suite passed throughout, because it used small ordinary values.

### Highlights

- **The invariant holds everywhere.** Every `Positive`-returning path now
  validates. Under `non-zero`, `1e-28 * 1e-28`, `0.5.floor()`,
  `0.4.round()` and `0.5.round_to(0)` each used to return `Positive(0)`.
- **Checked arithmetic no longer panics.** `checked_div` used raw
  division and aborted on `Decimal::MAX / 1e-28`; `checked_sub`,
  `sub_or_zero`, `sub_or_none` and `saturating_sub` used raw subtraction.
- **Serialisation is lossless.** A 28-digit fraction lost twelve digits
  through `f64`, and any integer above `i64::MAX` failed to serialise at
  all.
- **Comparison is lawful.** `positive == decimal` and `decimal ==
  positive` could disagree, comparison panicked at the extremes of
  `Decimal`'s range, and a finite float below `Decimal`'s smallest step
  underflowed to zero during conversion, so `Positive::ZERO == 1e-100`
  was `true`. Such floats are now ordered by sign.
- **Zero `unsafe`,** enforced by `#![forbid(unsafe_code)]`.

### Added

- `Positive::checked_add`, `checked_mul`, `checked_rem`, and the mixed
  `Decimal` family `checked_add_dec`, `checked_sub_dec`,
  `checked_mul_dec`, `checked_div_dec` (#71).
- `Positive::checked_sum`, generic over `Borrow<Positive>`, so owned and
  borrowed iterators share one non-panicking aggregation entry point
  (#72).
- Checked variants for every fallible mathematical operation:
  `checked_floor`, `checked_round`, `checked_round_to`,
  `checked_ceiling`, `checked_sqrt`, `checked_exp`, `checked_powi`,
  `checked_powu`, `checked_powd`, `checked_pow`, `checked_ln`,
  `checked_log10`, `checked_round_to_nice_number` (#73).
- `Positive::checked_format_fixed_places` (#81) and
  `Positive::checked_clamp` (#82).
- `Positive::approx_eq_dec` — the explicit, caller-supplied-tolerance
  replacement for the epsilon comparison `==` used to perform implicitly
  (#77).
- `Positive::is_multiple_of_within` — likewise for multiplicity (#78).
- `Positive::MAX` and `constants::MAX` (#76);
  `Positive::DAYS_IN_A_YEAR` (#84).
- `TryFrom<Positive>` for `u64`, `i64` and `usize` (#74).
- `PartialOrd<Positive> for Decimal`, so ordering exists in both
  directions (#77).
- `tests/boundary_matrix.rs`: a deterministic matrix over zero, `1e-28`,
  either side of one, `2^53 ± 1`, the integer limits, `Decimal::MAX`/`MIN`
  and a full 28-digit fraction, asserting that no checked API unwinds,
  every `Positive`-returning API upholds the invariant, serde round-trips
  exactly, and comparison is symmetric and consistent (#88).
- `make check`, a non-mutating quality gate mirroring
  `rules/global_rules.md`, run verbatim by CI (#86); `make lint-strict`
  (#83); `make audit` (#85).
- `rust-version = "1.85"`, verified in CI by building the library at
  exactly that version (#87).
- A tracked `LICENSE` file, and `rules/global_rules.md` under version
  control.

### Changed — breaking

- **`ln` and `log10` return `Decimal`, not `Positive`** (#73). The
  logarithm of a positive number is not necessarily positive; the return
  type was the defect.
- **serde emits the exact decimal as a string** — `"42.5"` rather than
  `42.5` (#75). Deserialisation still accepts the old numeric form.
- **`PositiveError::Other` is removed**, along with its `From<&str>` /
  `From<String>` constructors; `OutOfBounds` carries `Decimal` rather
  than `f64`; `InvalidValue.value` is a `String`;
  `InvalidPrecision.precision` is `u32`; `FromStr::Err` is
  `PositiveError` rather than `String` (#80).
- **`From<Positive> for u64` and `From<Positive> for usize` are
  removed** — they returned `0` when the value did not fit (#74).
- **`Positive::new_unchecked` is removed** (#79).
- `==` against `Decimal` and against `f64` is exact; the implicit epsilon
  is gone (#77). `is_multiple_of` is exact (#78).
- `clamp` takes `self` by value and panics on an inverted range (#82).
- `Display`, `Debug` and serde report the value `MAX` actually holds
  instead of `f64::MAX` (#76).

### Deprecated

Removal is scheduled for the release after 0.6.0.

- `Positive::INFINITY` and `constants::INFINITY` — use `MAX` (#76).
- `saturating_sub` — saturating arithmetic hides underflow (#71).
- `sqrt_checked` — renamed `checked_sqrt` (#73).
- `to_i64`, `to_u64`, `to_usize` — panic for valid values out of range;
  use `TryFrom` (#74).
- `to_f64_lossy` — `to_f64` is infallible and identical (#74).
- `is_multiple` — use `is_multiple_of_dec` (#78).

### Fixed

- `checked_div(Decimal::MAX, 1e-28)` panicked inside rust_decimal (#71).
- `Sum` folded with raw addition and applied `unwrap_or(ZERO)`, which
  could never observe the overflow it was meant to catch and would have
  replaced a financial total with zero (#72).
- `round_to_nice_number` panicked on zero and produced an invalid
  intermediate for every input below ten (#70, #73).
- `TryFrom<usize>` converted through `f64`, rounding every value above
  `2^53` (#74).
- `format_fixed_places` passed its argument straight to `format!`, so
  `u32::MAX` requested a four-billion-character string (#81); and even a
  valid precision of 28 overflowed `Decimal`'s internal formatting buffer
  for a 29-digit value (#88).
- Comparison and the `approx` implementations panicked at the extremes of
  `Decimal`'s range (#71, #77).
- Four RustSec advisories, and the Security Audit workflow, which had
  failed every scheduled run since 2026-08-05 and had additionally been
  disabled by GitHub for inactivity (#85).

### Migration

```rust
// ln / log10 now return Decimal (#73)
- let l: Positive = value.ln();
+ let l: Decimal = value.ln();

// serde: the wire format is a string (#75). Old documents still load;
// re-serialising upgrades them.
- {"price": 42.5}
+ {"price": "42.5"}

// error contract (#80)
- let e: String = "x".parse::<Positive>().unwrap_err();
+ let e: PositiveError = "x".parse::<Positive>().unwrap_err();
- if let PositiveError::OutOfBounds { value, .. } = e { let v: f64 = value; }
+ if let PositiveError::OutOfBounds { value, .. } = e { let v: Decimal = value; }

// integer conversions (#74)
- let n: u64 = positive.into();      // returned 0 when out of range
+ let n: u64 = u64::try_from(positive)?;

// unchecked construction (#79)
- let v = unsafe { Positive::new_unchecked(dec!(5.0)) };
+ let v = Positive::new_decimal(dec!(5.0))?;

// constant rename (#76)
- Positive::INFINITY
+ Positive::MAX

// approximate comparison is now explicit (#77, #78)
- if positive == some_decimal { .. }                       // was: within 1e-14
+ if positive.approx_eq_dec(some_decimal, EPSILON_CMP) { .. }
```

### Housekeeping

Folds in the unreleased 0.5.1, which bumped the optional `utoipa`
dependency from 5.4 to 5.5 and dropped a stale comment from
`Cargo.toml`, and was never given a changelog entry of its own.

## [0.5.0] - 2026-04-15

Major release completing milestones M2 through M7 of the performance
and rule-compliance programme. Contains several **breaking** API
changes (listed under *Removed* / *Changed*). Highlights below.

### Highlights

- **Hot paths faster by 10–80%.** See *Benchmarks vs 0.4.2* in the
  release notes: `format_fixed_places` −40 to −78%, `<Op><f64>` −18
  to −39%, `Positive → usize` −57%, many conversions −10 to −25%.
- **Invariant-safe arithmetic.** Every operator now routes through
  `Decimal::checked_*` and uniform `overflow_panic` /
  `invariant_panic` helpers (rules 50 / 52 / 63).
- **Rule-compliant surface.** `#[repr(transparent)]`, derive reorder,
  `#[cold]` on every error constructor, `#[inline]` on every hot-path
  helper, `#[must_use]` audit.
- **New checked `f64` API:** `checked_add_f64`, `checked_sub_f64`,
  `checked_mul_f64`, `checked_div_f64`, `checked_div_with_strategy`.
- **Documented `Div` rounding strategy** (`DIV_ROUNDING_STRATEGY`
  = banker's rounding) with per-call override.

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
  Full analysis lives in `doc/niche-optimization-proposal.md`
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
