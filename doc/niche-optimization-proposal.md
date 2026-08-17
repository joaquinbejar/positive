# Niche optimisation for `Option<Positive>` — proposal

**Issue:** #33
**Status:** Deferred pending benchmark evidence.
**Last updated:** 2026-04-15

---

## 1. Problem

`Positive` is a `#[repr(transparent)]` newtype over `rust_decimal::Decimal`.
`Decimal` internally holds four `u32` fields (mantissa lo/mid/hi + flags)
— none of them carry an unused bit pattern that the compiler can use as
a niche. As a result:

```text
size_of::<Positive>()         == 16 bytes
size_of::<Option<Positive>>() == 20 bytes (actually 24 after alignment)
```

Every `Option<Positive>` pays a discriminant byte that cannot be folded
into the value itself. For consumers that build dense arrays / maps
keyed by `Option<Positive>` this is a measurable memory overhead.

A niche-carrying variant would restore the optimisation:

```text
size_of::<PositiveNonZero>()         == 16 bytes
size_of::<Option<PositiveNonZero>>() == 16 bytes  (no discriminant)
```

## 2. Constraints

- Rule 44: `Decimal` is the only permitted storage. No pivot to
  `NonZeroU128` alone — we would lose `Decimal`'s scale field and
  precision guarantees.
- Rule 118: zero-`unsafe` target; any `unsafe` goes through the existing
  `new_unchecked` hatch with a documented `// SAFETY:` comment.
- Rule 184: semver stability on the existing `Positive` type.

## 3. Options considered

### Option A — Sibling type `PositiveNonZero`

Add a separate type built on top of `Decimal` but with an internal
representation designed to carry a niche (e.g. a `NonZeroU128` for the
mantissa + a `u8` for scale, reconstructing `Decimal` on demand).

- **Pros:** opt-in; existing `Positive` unchanged; downstream code can
  migrate where the niche matters.
- **Cons:** parallel API surface (constructors, arithmetic,
  conversions, serde) roughly doubling the crate; every arithmetic call
  pays reconstruction cost.
- **Semver:** additive, minor bump. Requires substantial engineering.

### Option B — Pivot `Positive` storage to a niche-carrying layout

Replace `Decimal` as the inner field with a custom packed struct that
wraps `NonZeroU128` + `u8` scale + sign-known-positive invariant.
Implement every arithmetic op by reconstructing a `Decimal`,
operating, then storing back.

- **Pros:** no new type; existing `Option<Positive>` automatically gets
  the niche.
- **Cons:** ABI break; `#[repr(transparent)]` over `Decimal` is lost
  (rule 23 violation as currently worded); every arithmetic op pays
  reconstruction cost; serde and `Into<Decimal>` change semantics for
  the zero case under `non-zero` (though that's the whole point). Large
  surface area to re-validate.
- **Semver:** major bump — not compatible with 0.5.x.

### Option C — Defer until benchmark evidence justifies it

Keep `Positive` as-is. Revisit when Criterion data or a downstream
profile shows `Option<Positive>` is actually a hot-path pain point.

## 4. Benchmark evidence (as of 2026-04-15)

None of the current Criterion suites (`arith`, `conversion`,
`format_serde`) exercises `Option<Positive>` directly. Cold data:

- `size_of::<Option<Positive>>()` is 24 bytes on x86-64.
- No downstream crate has reported a perf complaint.
- v0.5.0 refactors focused on per-op cost, not container size.

## 5. Recommendation

**Defer (Option C).** Revisit if and when:

1. A Criterion benchmark exists that exercises `Option<Positive>` in a
   realistic pattern.
2. Benchmarks show the discriminant byte costs more than 2–3% of
   total time (measurement must isolate the niche effect from normal
   cache behaviour).
3. A concrete downstream use case drives the design decision between
   Option A (sibling type) and Option B (layout pivot).

Neither condition holds today. Implementing either option now would be
speculative design.

## 6. Next actions if reopened

- Add a Criterion bench `benches/option_positive.rs` that exercises
  `Vec<Option<Positive>>` insertion, lookup, and bulk arithmetic.
- Add a memory-pressure scenario (e.g. 10 M `Option<Positive>` entries)
  comparing peak RSS against a baseline that uses `Positive` directly.
- Prototype **Option A** first (lower blast radius) and measure.
