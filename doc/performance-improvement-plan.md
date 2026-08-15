# Performance improvement plan — `positive` crate

**Analyzed version:** 0.4.0
**Date:** 2026-04-14
**Compliance:** This plan is aligned with `rules/global_rules.md`. Any suggestion that would violate those rules is rejected explicitly in § 0.

---

## Executive summary

`positive` is a `Copy + Hash` newtype around `rust_decimal::Decimal` guaranteeing positivity. The code is safe and correct, but several rule-compliant opportunities exist:

1. Missing `[profile.release]` in `Cargo.toml` (no LTO, default codegen units).
2. Missing `#[inline]` on small hot-path helpers — rule 14 explicitly calls for them.
3. Missing `#[cold]` on error-construction paths — rule 20.
4. Missing `#[repr(transparent)]` on `Positive` — rule 23.
5. Public inner field `Positive(pub Decimal)` violates rule 40 (invariant defense).
6. `.expect(...)` in operator impls and several methods violate rule 63.
7. Raw `+`, `-`, `*`, `/` on inner `Decimal` bypass `checked_*` — violates rule 50.
8. Division operators do not document a rounding strategy — rule 54.
9. `Decimal → f64 → Decimal` round-trips in `Mul<f64>`, `Div<f64>`, etc. — wastes cycles and loses precision.
10. `format_fixed_places` allocates via `format!` over an `f64` round-trip — rule 142 (no `String` in hot paths).
11. Custom `Serialize`/`Deserialize` duplicates validation — rule 32 suggests `#[serde(transparent)]`.
12. No benchmarks — rule 142 ("benchmark before claiming perf wins") requires them.

The plan is **6 phases**, each gated by `make pre-push` (rule 166).

---

## 0. Rule-compliance boundaries (read first)

The following ideas were considered and **rejected** to stay within `global_rules.md`:

| Idea | Why rejected | Rule |
|------|--------------|------|
| `unsafe fn add_unchecked(...)` to skip validation | Goal is **zero** `unsafe` in the crate. `new_unchecked` is the only exception and should not grow. | 118 |
| `Positive(self.0 + rhs)` without `checked_add` | Raw `+` on `Decimal` panics on overflow with no typed error. All arithmetic must go through `checked_*`. | 50 |
| `.expect("...")` inside operator impls | ZERO `.expect()` / `.unwrap()` in `src/` outside `#[cfg(test)]`. Operators may panic, but via an explicit `match` over the `checked_*` result that calls `panic!` with a formatted message (not via `.expect()`). | 63 |
| `panic = "abort"` in the release profile | Breaks consumer `catch_unwind`, and the operator contract is to panic on overflow. Not a free win. | 184 (semver stability) |
| Remove `#[must_use]` from pure methods to reduce noise | `#[must_use]` is mandatory on every pure function and `Result`/`Positive` returner. | 9–11 |
| Add `println!`/`log` timing calls in hot paths to profile | Library must not log. Use Criterion only. | 107 |
| Public accessor returning `&mut Decimal` | Direct rule violation — would break the positivity invariant. | 40, 183 |

Everything below sits inside these boundaries.

---

## Phase 0 — Benchmark harness (blocking prerequisite)

**Rule 142:** no perf claim without numbers. Nothing else ships before this.

### 0.1 Add Criterion as a dev-dependency

`Cargo.toml`:

```toml
[dev-dependencies]
serde_json = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "arith"
harness = false

[[bench]]
name = "conversion"
harness = false

[[bench]]
name = "format_serde"
harness = false
```

### 0.2 `benches/arith.rs`

Cover: `Add`, `Sub`, `Mul`, `Div` between `Positive`s; `Mul<f64>`, `Add<f64>`, `Sub<f64>`, `Div<f64>`; `sqrt`, `ln`, `exp`, `log10`; `round_to`; `clamp`; `sub_or_zero`, `saturating_sub`, `checked_sub`, `checked_div`, `is_multiple_of`.

### 0.3 `benches/conversion.rs`

Cover: `From<i64/u64/f64>`, `TryFrom<&Decimal>`, `Positive → f64/u64/usize/Decimal`, `FromStr`.

### 0.4 `benches/format_serde.rs`

Cover: `Display`, `Debug`, `format_fixed_places`, serde JSON (round-trip for integer, fractional, and `Positive::INFINITY`).

### 0.5 Freeze baseline

```bash
cargo bench -- --save-baseline v0.4.0
```

### 0.6 Acceptance gate

```bash
make pre-push
cargo bench
```

No phase progresses until this lands on main.

---

## Phase 1 — Rule-compliance fixes that are also perf wins

These changes make the code match `global_rules.md` **and** improve runtime. Zero behavioral changes to valid inputs.

### 1.1 Release profile

`Cargo.toml`:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3
debug = false
strip = true
# panic = "abort"   # intentionally omitted — see § 0
```

**Rationale:** LTO lets the compiler inline across the `rust_decimal` boundary, which is where most cycles actually live.

### 1.2 `#[repr(transparent)]` on `Positive`

`src/positive.rs:32-34`:

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Positive(Decimal);   // field now private — see § 1.3
```

Rules 23 and 30 require `#[repr(transparent)]` plus deriving the comparison traits. `Debug` stays as a manual `impl` to keep the `INFINITY` display, so the derive list skips it. `Eq`/`PartialOrd`/`Ord` can now come from derives — the manual impls at `src/positive.rs:1029-1055` become redundant and must be deleted.

### 1.3 Make the inner field private (**breaking**)

`src/positive.rs:34` is `pub struct Positive(pub Decimal);`. Rule 40 requires the field to be private. This is a breaking API change — schedule for v0.5.0, not a patch.

Migration:

- Replace `pub Decimal` with `Decimal`.
- Audit internal callers using `.0` directly and route them through `to_dec()` / `new_decimal()` / `new_unchecked()`.
- Document in `CHANGELOG.md`: "BREAKING: `Positive`'s inner field is now private. Use `to_dec()` to read the `Decimal`."

### 1.4 `#[cold]` on error constructors

`src/error.rs` — every `PositiveError::*` variant constructor (e.g. `invalid_value`, `arithmetic_error`, `out_of_bounds`, `conversion_error`, `invalid_precision`) must be `#[cold]` and `#[inline(never)]`. Rule 20.

Pattern:

```rust
#[cold]
#[inline(never)]
#[must_use]
pub fn arithmetic_error(op: &'static str, msg: impl Into<String>) -> Self { ... }
```

**Impact:** compiler avoids bloating hot call sites with the error-formatting code.

### 1.5 `#[inline]` on small helpers

`src/positive.rs` — annotate every function ≤ 3 lines in the hot path:

- `value()`, `to_f64()`, `to_dec()`.
- `is_zero()` (line 443).
- `round_to()` (line 412).
- `ln()` (406), `exp()` (425), `log10()` (455), `ceiling()` (449).
- `new_unchecked()` (554) — already `const`; add `#[inline]`.
- `impl From<Positive> for Decimal` (559), `impl From<&Positive|Positive> for f64` (577, 583), `impl From<Positive> for u64` (571), `impl From<&Positive> for Positive` (736).
- `impl PartialEq/PartialOrd<f64> for Positive/&Positive` (777–799).
- `impl Add/Sub/Div/Mul` between `Positive`s (936–967, 1064–1075).
- `impl AddAssign`, `impl MulAssign` (997–1013).

Rule 14 restricts `#[inline(always)]` to benchmark-proven hot paths. Do **not** use it blindly; wait for Criterion results from § 0.

### 1.6 `#[must_use]` audit

Rule 9. Every pure method on `Positive` returning a `Positive`, `Option<Positive>`, `Result<Positive, _>`, `Decimal`, `f64`, `bool`, or `String` must carry `#[must_use]`. Grep for `pub fn` in `src/positive.rs` and fix any that are missing it (`round_to`, `ln`, `exp`, `log10`, `ceiling`, `format_fixed_places`, `is_multiple`, `is_multiple_of`, `is_zero`, `clamp`, `checked_sub`, `checked_div`, etc. — verify all).

### 1.7 `From<Positive> for usize` without `f64`

`src/positive.rs:589-593`:

```rust
impl From<Positive> for usize {
    fn from(value: Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0) as usize   // lossy, rule-violating
    }
}
```

Replace with:

```rust
impl From<Positive> for usize {
    #[inline]
    #[must_use]
    fn from(value: Positive) -> Self {
        value.to_dec().to_u64().unwrap_or(0) as usize
    }
}
```

`to_u64()` returning `None` in pathological cases cannot be a silent `0`. Consider making the conversion `TryFrom<Positive> for usize` in v0.5.0; for v0.4.x preserve behavior but fix the `f64` round-trip.

### 1.8 `EPSILON_CMP` constant

`src/positive.rs:837` recomputes `EPSILON * Decimal::from(100)` on every `PartialEq<Decimal>` call. Promote it to a `const` in `src/constants.rs`:

```rust
pub const EPSILON_CMP: Decimal = /* precomputed value */;
```

And use it in the equality impl. Trivial win, cleaner hot path.

### 1.9 Gate

`make pre-push` must pass. Bench against baseline. Document results.

---

## Phase 2 — `checked_*` arithmetic (rule 50) and panic-via-match (rule 63)

Rule 50 is non-negotiable: **all** arithmetic goes through `checked_*`. Operators are allowed to panic (rule 52), but not via `.expect()`.

### 2.1 Canonical panic helper

Introduce one `#[cold] #[inline(never)]` helper in `src/positive.rs`:

```rust
#[cold]
#[inline(never)]
fn overflow_panic(op: &'static str) -> ! {
    panic!("Positive arithmetic overflow in {op}")
}

#[cold]
#[inline(never)]
fn invariant_panic(op: &'static str) -> ! {
    panic!("Positive invariant broken in {op}: result would be non-positive")
}
```

### 2.2 Rewrite every operator

Pattern for `Add`:

```rust
impl Add for Positive {
    type Output = Positive;
    #[inline]
    fn add(self, rhs: Positive) -> Positive {
        match self.to_dec().checked_add(rhs.to_dec()) {
            Some(v) => Positive(v),         // addition of two positives stays positive
            None => overflow_panic("add"),
        }
    }
}
```

Pattern for `Sub` (invariant must hold):

```rust
impl Sub for Positive {
    type Output = Positive;
    #[inline]
    fn sub(self, rhs: Positive) -> Positive {
        let r = self.to_dec().checked_sub(rhs.to_dec())
            .unwrap_or_else(|| overflow_panic("sub"));
        match Positive::new_decimal(r) {
            Ok(v) => v,
            Err(_) => invariant_panic("sub"),
        }
    }
}
```

Apply to: `Add` (936), `Sub` (943), `Mul` (1064), `Div` (955), the mixed `Decimal` ops (969–994, 1015–1027, 1071–1075), `AddAssign`/`MulAssign` (997–1013).

This removes every `.expect(...)` in the operator surface (rule 63) **and** routes through `checked_*` (rule 50).

### 2.3 `Mul<f64>` / `Div<f64>` / `Add<f64>` / `Sub<f64>` — stop round-tripping

`src/positive.rs:742-775` currently does `Decimal → f64 → operate → f64 → Decimal`. Rewrite:

```rust
impl Mul<f64> for Positive {
    type Output = Positive;
    #[inline]
    fn mul(self, rhs: f64) -> Positive {
        let rhs_dec = Decimal::from_f64(rhs)
            .unwrap_or_else(|| invariant_panic("mul_f64"));
        match self.to_dec().checked_mul(rhs_dec) {
            Some(v) if is_valid_positive_value(v) => Positive(v),
            Some(_)  => invariant_panic("mul_f64"),
            None     => overflow_panic("mul_f64"),
        }
    }
}
```

**Do not** assume `rhs >= 0`; validate through `is_valid_positive_value` and the `checked_mul`. This is correct regardless of feature flag (`non-zero` or not).

Offer **checked variants** as public methods so callers can opt out of panicking (rule 52 "checked path must exist"):

```rust
impl Positive {
    #[must_use]
    pub fn checked_mul_f64(self, rhs: f64) -> Result<Positive, PositiveError> { ... }
}
```

Apply to `Mul<f64>`, `Div<f64>`, `Add<f64>`, `Sub<f64>`.

### 2.4 Document division rounding (rule 54)

Rule 54 demands explicit, documented rounding on division. Today `/` on `Decimal` uses the default precision. Options:

- Pick `RoundingStrategy::MidpointNearestEven` (banker's rounding) and add to every `Div` impl:
  ```rust
  self.to_dec().checked_div(rhs.to_dec())
      .map(|v| v.round_dp_with_strategy(28, RoundingStrategy::MidpointNearestEven))
  ```
- Document it in the doc-comment of every `Div` impl.
- Add `checked_div_with_strategy` on `Positive` so callers can override.

### 2.5 `Neg` for `Positive` (src/positive.rs:1057-1062)

Currently `panic!("Cannot negate a Positive value!")`. Keep the panic (rule 52), but route it through `invariant_panic("neg")` for consistent messaging. Better: consider deleting `impl Neg` entirely (it is a trap); callable `Neg` on a type that can never satisfy it is arguably an API smell. Defer to v0.5.0.

### 2.6 Gate

`make pre-push`. Criterion delta vs Phase 1 baseline. Target: significant (>15%) win on `Mul<f64>` / `Div<f64>` loops.

---

## Phase 3 — Formatting and serde allocations

### 3.1 `format_fixed_places` without the `f64` round-trip

`src/positive.rs:418-421`:

```rust
// before
let rounded = self.round_to(decimal_places).to_f64();
format!("{:.1$}", rounded, decimal_places as usize)

// after
#[must_use]
pub fn format_fixed_places(&self, decimal_places: u32) -> String {
    format!("{:.1$}", self.to_dec().round_dp(decimal_places), decimal_places as usize)
}
```

Better precision, no `f64` step. Still allocates (unavoidable for `String` return), but only once.

### 3.2 Consider `#[serde(transparent)]` (rule 32)

Rule 32 prefers `#[serde(transparent)]` for single-field newtypes. Today `src/positive.rs:841-934` hand-rolls `Serialize`/`Deserialize` to:

- Emit `i64` when `scale == 0` and `f64` otherwise.
- Serialize `Positive::INFINITY` as `f64::MAX`.
- Validate on deserialize.

If we adopt `#[serde(transparent)]`:

- Pros: zero duplication, delegates to `rust_decimal`'s own serde impl, simpler.
- Cons: loses the `INFINITY → f64::MAX` projection and the `i64`/`f64` output split. JSON consumers relying on those shapes break.

**Recommendation:** evaluate against real wire-format compatibility needs. If the current JSON shape is load-bearing for downstream services, **keep the manual impl** and just remove the duplicated validation (§ 3.3). If not, switch to `#[serde(transparent)]` in v0.5.0 with a `CHANGELOG.md` entry.

### 3.3 Remove duplicated validation in the serde visitor

`src/positive.rs:888-929`: `visit_i64`, `visit_u64`, `visit_f64` each check `is_valid_positive_value(decimal)` and then call `Positive::new_decimal`, which re-validates. Drop the first check:

```rust
fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
    Positive::new_decimal(Decimal::from(value)).map_err(E::custom)
}
```

### 3.4 `Display` / `Debug` allocation

`src/positive.rs:801-818` builds a `String` and double-trims. Try `write!(f, "{}", self.0.normalize())` — `rust_decimal::Decimal::normalize` drops trailing zeros without allocating a `String`. Validate all Display tests pass; if any edge case breaks (e.g. `0` vs `0.0` formatting), keep the manual path.

### 3.5 Gate

`make pre-push`. Bench `format_fixed_places` and serde round-trip.

---

## Phase 4 — Correctness fixes with perf relevance

### 4.1 `is_multiple` should not go through `f64`

`src/positive.rs:514-523`:

```rust
// before — loses precision via f64
pub fn is_multiple(&self, other: f64) -> bool { ... }

// after — true decimal arithmetic
#[must_use]
pub fn is_multiple_of_dec(&self, other: Decimal) -> bool {
    if other.is_zero() { return false; }
    self.to_dec()
        .checked_rem(other)
        .map(|r| r.is_zero())
        .unwrap_or(false)
}
```

Keep `is_multiple(&self, f64)` as a thin deprecated wrapper for semver (rule 184): do not rename/remove in v0.4.x.

### 4.2 `is_multiple_of` already uses `Decimal`

`src/positive.rs:527-533` uses `self.0 % other.0`. Replace with `checked_rem` (rule 50) to avoid a panic on edge cases.

### 4.3 Gate

`make pre-push`.

---

## Phase 5 — Optional: const-ness audit

### 5.1 Constants in `src/constants.rs`

Ensure every constant is built via `dec!(...)` so it is evaluated at compile time. Anything currently computed at runtime should become a `const`.

### 5.2 `new_unchecked` stays as the only `unsafe`

Do **not** add more `unsafe` (rule 118 — target is zero). Improve its doc-comment so callers understand the invariant they must uphold.

---

## Phase 6 — Deferred design questions (post-v0.4.x)

These require user approval (rule 177) and/or semver-major bumps.

1. **Niche optimization for `Option<Positive>`.** `Decimal` has no niche, so `Option<Positive>` pays a discriminant byte. A separate `PositiveNonZero` using `NonZeroU128` + scale would recover the niche but requires invasive redesign. Only if benchmarks show `Option<Positive>` is a pain point downstream.
2. **`PositiveF64` sibling type.** 5–10× faster arithmetic but gives up decimal precision — contradicts rule 44 ("`Decimal` only"). Reject unless the user explicitly creates a separate crate.
3. **Remove `impl Neg for Positive`.** See § 2.5.

---

## Execution order and gating

1. **Phase 0** — Criterion + baseline.
2. **Phase 1** — rule-compliance + inlining + profile.
3. **Phase 2** — `checked_*` + panic helper + `f64` op rewrite.
4. **Phase 3** — formatting / serde.
5. **Phase 4** — `is_multiple` correctness.
6. **Phase 5** — const-ness audit.
7. **Phase 6** — deferred.

Between phases:

```bash
make pre-push
cargo bench -- --baseline v0.4.0
```

Update `CHANGELOG.md` with each phase. Keep the README in sync (rule 117) via `make readme`.

---

## Acceptance criteria per phase

All of these must hold after every phase (rule 146-170):

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
- `cargo test --all-features`
- `cargo test --no-default-features`
- `cargo test --features non-zero`
- `cargo build --release` with zero warnings
- `cargo doc --no-deps --all-features` with zero warnings
- Zero `.unwrap()` / `.expect()` / unchecked `[]` in `src/` outside `#[cfg(test)]`
- `#[must_use]` on every pure public function
- `#[inline]` on small hot-path helpers; `#[cold]` on error paths; `#[inline(always)]` only where Criterion proved it matters
- `rust_decimal::Decimal` remains the only storage
- Invariant holds under default AND `non-zero`
- Criterion shows a measurable win vs the previous phase's baseline — or the phase is reverted

---

## Risks

| Risk | Mitigation |
|------|------------|
| Making the inner field private (§ 1.3) breaks downstream code | Schedule for v0.5.0, not a patch. Document clearly in CHANGELOG; provide `to_dec()` as the migration path. |
| `#[serde(transparent)]` (§ 3.2) changes JSON wire format | Keep manual impl for v0.4.x; only migrate if a semver-major bump is approved. |
| `Decimal::normalize()` in `Display` (§ 3.4) shifts output for edge cases | Snapshot-test Display for: integer, fractional, trailing-zero, `ZERO`, `ONE`, `INFINITY`, very small values. Revert if any change. |
| Criterion showing < 5 % win on a phase | Revert that phase. Do not merge perf work without measured benefit (rule 142). |
| `#[inline]` bloating binary size | Measure with `cargo bloat`; prefer `#[inline]` over `#[inline(always)]`. |

---

## Files touched (summary)

- `Cargo.toml` — `[profile.release]`, Criterion dev-dep, `[[bench]]` entries.
- `src/positive.rs` — `#[repr(transparent)]`, derive ordering, private field (v0.5.0), `#[inline]`, `#[must_use]`, `checked_*` operators, panic helpers, `f64` ops rewrite, `Display` / `Debug` / `format_fixed_places`, `From<Positive> for usize`, `is_multiple`.
- `src/error.rs` — `#[cold] #[inline(never)]` on constructors.
- `src/constants.rs` — `EPSILON_CMP`.
- `benches/arith.rs`, `benches/conversion.rs`, `benches/format_serde.rs` — new.
- `CHANGELOG.md` — one section per phase, breaking changes flagged.
- `README.md` / `src/lib.rs` — regenerated if crate docs changed (`make readme`).
