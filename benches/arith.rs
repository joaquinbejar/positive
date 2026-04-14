//! Arithmetic benchmarks for the `Positive` type.
//!
//! Covers Positive-Positive operators, mixed Positive-f64 operators,
//! mathematical functions, rounding/clamping, subtraction variants, and
//! multiplicity checks. Values are chosen to avoid panics on the hot path
//! (e.g. `a > b` for `Sub`, non-zero divisors for `Div`/`checked_div`).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use positive::Positive;
use rust_decimal_macros::dec;

fn p(value: rust_decimal::Decimal) -> Positive {
    Positive::new_decimal(value).expect("bench input must be a valid Positive")
}

fn bench_arith_positive(c: &mut Criterion) {
    let a = p(dec!(12.5));
    let b = p(dec!(7.25));
    let mut g = c.benchmark_group("arith/positive");
    g.bench_function("add", |bencher| {
        bencher.iter(|| black_box(black_box(a) + black_box(b)))
    });
    g.bench_function("sub", |bencher| {
        bencher.iter(|| black_box(black_box(a) - black_box(b)))
    });
    g.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(black_box(a) * black_box(b)))
    });
    g.bench_function("div", |bencher| {
        bencher.iter(|| black_box(black_box(a) / black_box(b)))
    });
    g.finish();
}

fn bench_arith_f64(c: &mut Criterion) {
    let a = p(dec!(12.5));
    let rhs = 1.5_f64;
    let mut g = c.benchmark_group("arith/f64");
    g.bench_function("mul_f64", |bencher| {
        bencher.iter(|| black_box(black_box(a) * black_box(rhs)))
    });
    g.bench_function("div_f64", |bencher| {
        bencher.iter(|| black_box(black_box(a) / black_box(rhs)))
    });
    g.bench_function("add_f64", |bencher| {
        bencher.iter(|| black_box(black_box(a) + black_box(rhs)))
    });
    g.bench_function("sub_f64", |bencher| {
        bencher.iter(|| black_box(black_box(a) - black_box(rhs)))
    });
    g.finish();
}

fn bench_math(c: &mut Criterion) {
    let big = p(dec!(16.0));
    let small = p(dec!(1.5));
    let mut g = c.benchmark_group("math");
    g.bench_function("sqrt", |bencher| {
        bencher.iter(|| black_box(black_box(big).sqrt()))
    });
    g.bench_function("ln", |bencher| {
        bencher.iter(|| black_box(black_box(big).ln()))
    });
    g.bench_function("exp", |bencher| {
        bencher.iter(|| black_box(black_box(small).exp()))
    });
    g.bench_function("log10", |bencher| {
        bencher.iter(|| black_box(black_box(big).log10()))
    });
    g.finish();
}

fn bench_round_clamp(c: &mut Criterion) {
    let a = p(dec!(12.345678));
    let min = p(dec!(1.0));
    let max = p(dec!(10.0));
    let mut g = c.benchmark_group("round_clamp");
    g.bench_function("round_to_2", |bencher| {
        bencher.iter(|| black_box(black_box(a).round_to(black_box(2))))
    });
    g.bench_function("clamp", |bencher| {
        bencher.iter(|| black_box(black_box(a).clamp(black_box(min), black_box(max))))
    });
    g.finish();
}

fn bench_sub_variants(c: &mut Criterion) {
    let a = p(dec!(10.0));
    let b = p(dec!(3.0));
    let d = dec!(3.0);
    let mut g = c.benchmark_group("sub_variants");
    g.bench_function("checked_sub", |bencher| {
        bencher.iter(|| black_box(black_box(a).checked_sub(black_box(&b))))
    });
    #[cfg(not(feature = "non-zero"))]
    g.bench_function("sub_or_zero", |bencher| {
        bencher.iter(|| black_box(black_box(a).sub_or_zero(black_box(&d))))
    });
    #[cfg(not(feature = "non-zero"))]
    g.bench_function("saturating_sub", |bencher| {
        bencher.iter(|| black_box(black_box(a).saturating_sub(black_box(&b))))
    });
    #[cfg(feature = "non-zero")]
    let _ = d;
    g.finish();
}

fn bench_checked_div(c: &mut Criterion) {
    let a = p(dec!(10.0));
    let b = p(dec!(3.0));
    let mut g = c.benchmark_group("checked");
    g.bench_function("checked_div", |bencher| {
        bencher.iter(|| black_box(black_box(a).checked_div(black_box(&b))))
    });
    g.finish();
}

fn bench_is_multiple_of(c: &mut Criterion) {
    let a = p(dec!(15.0));
    let b = p(dec!(3.0));
    c.bench_function("is_multiple_of", |bencher| {
        bencher.iter(|| black_box(black_box(a).is_multiple_of(black_box(&b))))
    });
}

criterion_group!(
    benches,
    bench_arith_positive,
    bench_arith_f64,
    bench_math,
    bench_round_clamp,
    bench_sub_variants,
    bench_checked_div,
    bench_is_multiple_of,
);
criterion_main!(benches);
