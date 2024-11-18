use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Import StampFolder directly since we're using it from main.rs
use folds::cpu::StampFolder;

fn benchmark_small_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Small Dimensions");

    // Test small 2xN dimensions
    group.bench_function("2x2", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[2, 2]), 4);
        });
    });

    group.bench_function("2x3", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[2, 3]), 4);
        });
    });

    group.bench_function("2x4", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[2, 4]), 4);
        });
    });

    group.finish();
}

fn benchmark_medium_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Medium Dimensions");

    group.bench_function("3x3", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[3, 3]), 4);
        });
    });

    group.bench_function("3x4", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[3, 4]), 4);
        });
    });

    group.finish();
}

fn benchmark_large_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Dimensions");

    group.bench_function("4x4", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[4, 4]), 4);
        });
    });

    // Note: 5x5 might be too slow for regular benchmarking
    group.sample_size(10); // Reduce sample size for larger dimensions
    group.bench_function("5x5", |b| {
        b.iter(|| {
            StampFolder::calculate_sequence_parallel(black_box(&[5, 5]), 4);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_small_dimensions,
    benchmark_medium_dimensions,
    benchmark_large_dimensions
);
criterion_main!(benches);
