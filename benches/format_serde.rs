//! Formatting and serde benchmarks for the `Positive` type.
//!
//! Covers `Display`, `Debug`, `format_fixed_places`, and JSON round-trip for
//! integer-valued, fractional, very small, and `Positive::INFINITY` inputs.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn p(value: Decimal) -> Positive {
    Positive::new_decimal(value).expect("bench input must be a valid Positive")
}

fn bench_display(c: &mut Criterion) {
    let integer_valued = p(dec!(12345));
    let fractional = p(dec!(12345.6789));
    let very_small = p(dec!(0.0000001));
    let infinity = Positive::INFINITY;
    let mut g = c.benchmark_group("fmt/display");
    g.bench_function("integer", |bencher| {
        bencher.iter(|| black_box(format!("{}", black_box(integer_valued))))
    });
    g.bench_function("fractional", |bencher| {
        bencher.iter(|| black_box(format!("{}", black_box(fractional))))
    });
    g.bench_function("very_small", |bencher| {
        bencher.iter(|| black_box(format!("{}", black_box(very_small))))
    });
    g.bench_function("infinity", |bencher| {
        bencher.iter(|| black_box(format!("{}", black_box(infinity))))
    });
    g.finish();
}

fn bench_debug(c: &mut Criterion) {
    let integer_valued = p(dec!(12345));
    let fractional = p(dec!(12345.6789));
    let infinity = Positive::INFINITY;
    let mut g = c.benchmark_group("fmt/debug");
    g.bench_function("integer", |bencher| {
        bencher.iter(|| black_box(format!("{:?}", black_box(integer_valued))))
    });
    g.bench_function("fractional", |bencher| {
        bencher.iter(|| black_box(format!("{:?}", black_box(fractional))))
    });
    g.bench_function("infinity", |bencher| {
        bencher.iter(|| black_box(format!("{:?}", black_box(infinity))))
    });
    g.finish();
}

fn bench_format_fixed_places(c: &mut Criterion) {
    let value = p(dec!(12345.6789));
    let mut g = c.benchmark_group("fmt/format_fixed_places");
    for &places in &[0_u32, 2, 4, 8] {
        g.bench_function(format!("places_{places}"), |bencher| {
            bencher.iter(|| black_box(black_box(value).format_fixed_places(black_box(places))))
        });
    }
    g.finish();
}

fn bench_serde_roundtrip(c: &mut Criterion) {
    let integer_valued = p(dec!(12345));
    let fractional = p(dec!(12345.6789));
    let very_small = p(dec!(0.0000001));
    let infinity = Positive::INFINITY;
    let json_integer = serde_json::to_string(&integer_valued).expect("ser");
    let json_fractional = serde_json::to_string(&fractional).expect("ser");
    let json_small = serde_json::to_string(&very_small).expect("ser");
    let json_infinity = serde_json::to_string(&infinity).expect("ser");

    let mut g = c.benchmark_group("serde/to_string");
    g.bench_function("integer", |bencher| {
        bencher.iter(|| black_box(serde_json::to_string(black_box(&integer_valued))))
    });
    g.bench_function("fractional", |bencher| {
        bencher.iter(|| black_box(serde_json::to_string(black_box(&fractional))))
    });
    g.bench_function("very_small", |bencher| {
        bencher.iter(|| black_box(serde_json::to_string(black_box(&very_small))))
    });
    g.bench_function("infinity", |bencher| {
        bencher.iter(|| black_box(serde_json::to_string(black_box(&infinity))))
    });
    g.finish();

    let mut g = c.benchmark_group("serde/from_str");
    g.bench_function("integer", |bencher| {
        bencher.iter(|| black_box(serde_json::from_str::<Positive>(black_box(&json_integer))))
    });
    g.bench_function("fractional", |bencher| {
        bencher.iter(|| {
            black_box(serde_json::from_str::<Positive>(black_box(
                &json_fractional,
            )))
        })
    });
    g.bench_function("very_small", |bencher| {
        bencher.iter(|| black_box(serde_json::from_str::<Positive>(black_box(&json_small))))
    });
    g.bench_function("infinity", |bencher| {
        bencher.iter(|| black_box(serde_json::from_str::<Positive>(black_box(&json_infinity))))
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_display,
    bench_debug,
    bench_format_fixed_places,
    bench_serde_roundtrip,
);
criterion_main!(benches);
