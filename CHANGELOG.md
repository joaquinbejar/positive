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
