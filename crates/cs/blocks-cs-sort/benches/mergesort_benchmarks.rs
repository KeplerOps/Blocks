use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn mergesort_benchmark(_c: &mut Criterion) {
    // TODO: Implement mergesort benchmarks
}

criterion_group!(benches, mergesort_benchmark);
criterion_main!(benches);