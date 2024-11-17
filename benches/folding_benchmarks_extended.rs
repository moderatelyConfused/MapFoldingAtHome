use criterion::{black_box, criterion_group, criterion_main, Criterion};
use folds::StampFolder;

fn benchmark_rectangular(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rectangular Dimensions");
    
    // Test rectangular dimensions with constant width
    group.bench_function("2x5", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 5]), true, 0, 0);
        });
    });

    group.bench_function("2x6", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 6]), true, 0, 0);
        });
    });

    // Test rectangular dimensions with varying width
    group.bench_function("3x5", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[3, 5]), true, 0, 0);
        });
    });

    group.finish();
}

fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Area Comparison");
    
    // Compare different shapes with same area
    group.bench_function("2x6 (area=12)", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[2, 6]), true, 0, 0);
        });
    });

    group.bench_function("3x4 (area=12)", |b| {
        b.iter(|| {
            let mut folder = StampFolder::new();
            folder.foldings(black_box(&[3, 4]), true, 0, 0);
        });
    });

    group.finish();
}

criterion_group!(
    extended_benches,
    benchmark_rectangular,
    benchmark_comparison
);
criterion_main!(extended_benches);
