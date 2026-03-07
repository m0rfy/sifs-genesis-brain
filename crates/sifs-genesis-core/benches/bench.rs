//! Benchmarks: Day Phase + Night Phase for 1K, 10K, 100K neurons.
//! Compare with SIFS_Genesis_Hybrid_Report (~0.38 ms/step for 10K).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sifs_genesis_core::run_benchmark;

fn bench_1k(c: &mut Criterion) {
    c.bench_function("brain_1k_1000steps", |b| {
        b.iter(|| {
            black_box(run_benchmark(1000, 1000, 100));
        });
    });
}

fn bench_10k(c: &mut Criterion) {
    c.bench_function("brain_10k_500steps", |b| {
        b.iter(|| {
            black_box(run_benchmark(10_000, 500, 100));
        });
    });
}

fn bench_100k(c: &mut Criterion) {
    c.bench_function("brain_100k_100steps", |b| {
        b.iter(|| {
            black_box(run_benchmark(100_000, 100, 50));
        });
    });
}

criterion_group!(benches, bench_1k, bench_10k, bench_100k);
criterion_main!(benches);
