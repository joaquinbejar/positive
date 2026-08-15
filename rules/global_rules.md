Rules for writing Rust in the `positive` library — a type-safe wrapper for guaranteed positive decimal values used in financial and scientific calculations.
All code, comments, docs, commit messages, and PR descriptions in English.

---

## Compiler Attributes

### #[must_use]
- All pure functions (validation, parsing, conversions, arithmetic).
- Builder-like methods and constructors that return a `Result<Positive, PositiveError>` or `Option<Positive>`.
- Every method that returns a `Positive`, a `Result<Positive, _>`, or a numeric conversion — discarding them is almost always a bug.

### #[inline] / #[inline(always)] / #[inline(never)]
- `#[inline]`: small frequent functions — constructors, conversions, comparisons, arithmetic helpers.
- `#[inline(always)]`: ONLY for the tightest hot paths that benchmarks prove matter (e.g. repeated arithmetic in downstream callers). Do not over-use.
- `#[inline(never)]`: error construction, formatting, anything that would bloat call sites without benefit.
- No attribute: mid-size functions (10–50 lines).

### #[cold]
- Error construction helpers, validation failures, `PositiveError` variant builders, any path that only runs on invalid input.

### #[repr]
- `#[repr(transparent)]` on `Positive` since it wraps a single `Decimal` — enables zero-cost FFI and guarantees layout.
- `#[repr(u8)]` on any small enum (e.g. `PositiveError` discriminants when the representation matters for stability).

### #[derive] — exact order
```
Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default
```
Only derive what is needed. No `Ord` if ordering is meaningless. No `Default` if no sensible default exists. `Positive` intentionally derives `PartialOrd`/`Ord` because positive decimals are totally ordered.

Serde conventions:
- `#[serde(transparent)]` for single-field newtypes — `Positive` wraps `Decimal` transparently so JSON round-trips as a number.
- `#[serde(deny_unknown_fields)]` on any future config-like structs.

---

## Type Safety

- `Positive` wraps `rust_decimal::Decimal`. The inner field is **private** — this is the whole point of the type. Never expose it, never add a `pub` accessor that returns `&mut Decimal`.
- Constructors: `Positive::new()` / `Positive::new_decimal()` returning `Result<Self, PositiveError>`. Variants via macros: `pos!` (Result), `pos_or_panic!` (panics on invalid input — only for tests and examples), `spos!` (Option).
- With the `non-zero` feature: values must be strictly `> 0`. Without it: values are `>= 0`. Construction must enforce this invariant in every path.
- NEVER let a caller construct a `Positive` that violates the invariant. All conversions from `f64`, `i64`, `u64`, `&str`, `Decimal` must validate.
- Use `rust_decimal::Decimal` exclusively for the underlying representation. No `f64` storage.

---

## Arithmetic — Mandatory

- ALL arithmetic on the underlying `Decimal`: use `checked_add`, `checked_sub`, `checked_mul`, `checked_div`.
- NEVER use `saturating_*` or `wrapping_*` — they silently hide overflows, which is catastrophic in financial math.
- `checked_*` methods on `Positive` must return `Result<Positive, PositiveError>` and surface overflow as `ArithmeticError`. The operator overloads (`+`, `-`, `*`, `/`) must panic on overflow/underflow, but only after the checked path exists so callers can opt into the non-panicking form.
- Subtraction must preserve the `Positive` invariant — returning an error (or panicking from the operator) if the result would be negative (or zero under the `non-zero` feature).
- Every division: explicitly choose and document rounding (`RoundingStrategy::MidpointNearestEven`, truncate, etc.). Division by zero must return `ArithmeticError`, never panic silently.

---

## Error Handling

- `thiserror` for `PositiveError`. Variants are exhaustive and stable across minor versions: `InvalidValue`, `ArithmeticError`, `ConversionError`, `OutOfBounds`, `InvalidPrecision`.
- No `anyhow`. All fallible public functions return `Result<_, PositiveError>`.
- Error messages: lowercase, human-readable, include the offending value when possible.
- ZERO `.unwrap()`, `.expect()`, unchecked `[]` indexing in production code (i.e. inside `src/` excluding `#[cfg(test)]`). Examples under `examples/` may use `.unwrap()` / `pos_or_panic!` for brevity, but mark them clearly.
- `.get()` for slice/vec access, `.ok_or_else()` for `Option` → `Result`.
- `debug_assert!` for internal invariants during development.
- `assert!` ONLY for truly unrecoverable conditions (never in steady-state code — the constructor invariant is the only defense we should need).

---

## Minimize Copies

- `Positive` is `Copy` (it wraps `Decimal` which is `Copy`). Pass by value freely; prefer `Positive` over `&Positive` in arithmetic APIs.
- Pre-allocate collections with `Vec::with_capacity` when the size is known or estimable (constants tables, test vectors).
- Avoid `.clone()` when `Copy` suffices — lint yourself.

---

## Code Organization

- One concern per file:
  - `src/positive.rs` — the `Positive` struct, constructors, arithmetic, conversions
  - `src/constants.rs` — predefined constants (numeric, mathematical, special values)
  - `src/macros.rs` — `pos!`, `pos_or_panic!`, `spos!` and any helper macros
  - `src/error.rs` — `PositiveError` and its variants
  - `src/prelude.rs` — re-exports for `use positive::prelude::*;`
  - `src/tests.rs` / `#[cfg(test)] mod tests` — unit tests
  - `tests/` — integration tests
  - `examples/` — runnable examples demonstrating the API
- Re-export important types from `lib.rs` and `prelude.rs`. Keep the public surface intentional and documented.
- Constants: associated constants on `Positive` (e.g. `Positive::ONE`, `Positive::PI`) AND free-standing re-exports in the `constants` module. Keep both in sync. No magic numbers elsewhere.

---

## Feature Flags

- `default = []` — keep the default surface minimal and cheap.
- `non-zero` — changes the invariant from `>= 0` to `> 0`. Every constructor and arithmetic method must respect it. Tests must run under both modes where applicable.
- `utoipa` — optional, behind `dep:utoipa`. Never make the core depend on an optional dep.
- Adding a new feature requires explicit user approval and a note in `README.md` and `src/lib.rs`.

---

## Logging & Observability

This is a library, not a service. Do NOT pull in a logging framework by default.

- No `println!`, `eprintln!`, `dbg!`, or `log` crate calls in `src/`. Ever. The caller decides how to log.
- If, in the future, diagnostic output is genuinely needed, gate it behind a `tracing` feature flag using `tracing` only — never hardcoded `println!`.
- Examples under `examples/` may use `println!` to show output — that is their purpose.

---

## Documentation

- Every `pub` item: `///` doc comment with description, `# Errors` (for `Result`-returning APIs), `# Panics` (for `_or_panic` APIs and operators that panic on invariant violation), and `# Examples` for anything non-trivial.
- Include units in all numeric doc comments when relevant ("basis points", "seconds", "ratio", etc.).
- `README.md` and `src/lib.rs` crate docs must stay in sync — the README is generated from the crate docs via `make readme`. Update crate docs first.
- No `unsafe` blocks. If one is ever genuinely required, add a `// SAFETY:` comment explaining the invariant — but the target is zero `unsafe` in this crate.

---

## Testing

- Unit tests in the same file (`#[cfg(test)] mod tests`) or in `src/tests.rs` for cross-cutting tests.
- Integration tests in `tests/` for the public API surface.
- Every test covers both the happy path and all documented error cases, including the invariant: zero handling under both default and `non-zero` modes, overflow, underflow (subtraction going negative), division by zero.
- Use `rust_decimal_macros::dec!` for readable decimal literals in tests.
- Name tests as `test_<unit>_<scenario>_<expected>`, e.g., `test_positive_new_negative_returns_out_of_bounds`.
- Run the full suite under both feature configurations when you touch anything invariant-related:
  ```bash
  cargo test --all-features
  cargo test --no-default-features
  cargo test --features non-zero
  ```

---

## Performance — General

- `Positive` is `Copy` and `#[repr(transparent)]` — it must stay that way. Do not add fields that break the transparent layout without a very strong reason.
- Benchmark before claiming perf wins. No "it's faster" without numbers.
- Avoid allocations in arithmetic paths. `Decimal` arithmetic does not allocate; keep it that way by not introducing `String` formatting into hot methods.

---

## Pre-Submission Checklist

All must pass — failing any means not ready:

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
- `cargo test --all-features`
- `cargo test --no-default-features`
- `cargo test --features non-zero`
- `cargo build --release` (zero warnings)
- `cargo doc --no-deps --all-features` (zero warnings — public API docs complete)
- No `.unwrap()` / `.expect()` / unchecked indexing in `src/` (outside `#[cfg(test)]`)
- `#[must_use]` on all pure functions and constructors
- `#[inline]` on small hot-path helpers, `#[cold]` on error paths
- `rust_decimal::Decimal` underneath — no `f64` storage anywhere
- The `Positive` invariant holds under both default and `non-zero` features
- Tests cover happy path AND all error cases
- Doc comments on all `pub` items, with `# Errors` / `# Panics` / `# Examples` where applicable
- `README.md` regenerated if the crate docs changed (`make readme`)

The project shortcut runs most of this:

```bash
make pre-push
```

---

## DO NOT

- Use `todo!()` or `unimplemented!()` in submitted code.
- Add dependencies without explicit approval.
- Use `f64` as the storage representation for `Positive` — `Decimal` only.
- Use `anyhow` — `PositiveError` with `thiserror` is the one error type.
- Use `println!`, `eprintln!`, `dbg!`, or the `log` crate in `src/`.
- Skip any pre-submission check.
- Use `saturating_*` or `wrapping_*` arithmetic.
- Expose the inner `Decimal` field via a `pub` accessor that permits mutation — it would break the invariant.
- Break semver: do not rename public items, remove variants from `PositiveError`, or change method signatures in a non-major release.
