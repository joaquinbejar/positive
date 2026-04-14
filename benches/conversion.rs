//! Conversion benchmarks for the `Positive` type.
//!
//! Covers primitive-to-Positive (`new`, `TryFrom`), Positive-to-primitive
//! (`From<Positive>` for `Decimal`/`u64`/`f64`/`usize`), and `FromStr`.

use std::hint::black_box;
use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn p(value: Decimal) -> Positive {
    Positive::new_decimal(value).expect("bench input must be a valid Positive")
}

fn bench_from_primitive(c: &mut Criterion) {
    let d = dec!(9.99);
    let mut g = c.benchmark_group("conv/from_primitive");
    g.bench_function("new_f64", |bencher| {
        bencher.iter(|| black_box(Positive::new(black_box(3.25_f64))))
    });
    g.bench_function("try_from_i64", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(42_i64))))
    });
    g.bench_function("try_from_u64", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(42_u64))))
    });
    g.bench_function("try_from_f64", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(3.25_f64))))
    });
    g.bench_function("try_from_usize", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(100_usize))))
    });
    g.bench_function("try_from_decimal", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(d))))
    });
    g.bench_function("try_from_ref_decimal", |bencher| {
        bencher.iter(|| black_box(Positive::try_from(black_box(&d))))
    });
    g.finish();
}

fn bench_to_primitive(c: &mut Criterion) {
    let value = p(dec!(12345.6789));
    let mut g = c.benchmark_group("conv/to_primitive");
    g.bench_function("to_decimal", |bencher| {
        bencher.iter(|| black_box(Decimal::from(black_box(value))))
    });
    g.bench_function("to_u64", |bencher| {
        bencher.iter(|| black_box(u64::from(black_box(value))))
    });
    g.bench_function("to_f64", |bencher| {
        bencher.iter(|| black_box(f64::from(black_box(value))))
    });
    g.bench_function("to_usize", |bencher| {
        bencher.iter(|| black_box(usize::from(black_box(value))))
    });
    g.finish();
}

fn bench_from_str(c: &mut Criterion) {
    let short = "3.14";
    let long = "12345.678901234";
    let mut g = c.benchmark_group("conv/from_str");
    g.bench_function("short", |bencher| {
        bencher.iter(|| black_box(Positive::from_str(black_box(short))))
    });
    g.bench_function("long", |bencher| {
        bencher.iter(|| black_box(Positive::from_str(black_box(long))))
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_from_primitive,
    bench_to_primitive,
    bench_from_str,
);
criterion_main!(benches);
