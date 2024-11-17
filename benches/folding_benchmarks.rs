use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Import StampFolder directly since we're using it from main.rs
use folds::StampFolder;

fn benchmark_small_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Small Dimensions");
    
    // Test small 2xN dimensions
    group.bench_function("2x2", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 2]), true, 0, 0);
        });
    });

    group.bench_function("2x3", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 3]), true, 0, 0);
        });
    });

    group.bench_function("2x4", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 4]), true, 0, 0);
        });
    });

    group.finish();
}

fn benchmark_medium_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Medium Dimensions");
    
    group.bench_function("3x3", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[3, 3]), true, 0, 0);
        });
    });

    group.bench_function("3x4", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[3, 4]), true, 0, 0);
        });
    });

    group.finish();
}

fn benchmark_large_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Dimensions");
    
    group.bench_function("4x4", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[4, 4]), true, 0, 0);
        });
    });

    // Note: 5x5 might be too slow for regular benchmarking
    group.sample_size(10); // Reduce sample size for larger dimensions
    group.bench_function("5x5", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[5, 5]), true, 0, 0);
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