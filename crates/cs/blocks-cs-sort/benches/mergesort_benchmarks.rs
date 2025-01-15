use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn mergesort_benchmark(c: &mut Criterion) {
    // Placeholder for mergesort benchmarks
    c.bench_function("mergesort", |b| b.iter(|| 1));
}

criterion_group!(benches, mergesort_benchmark);
criterion_main!(benches);