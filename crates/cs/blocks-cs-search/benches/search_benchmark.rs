use blocks_cs_search::algorithms::{binary_search, linear_search};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};

fn bench_search_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_comparison");
    
    // Test different array sizes
    for size in [10, 100, 1000, 10000].iter() {
        let arr: Vec<i32> = (0..*size).collect();
        
        // Best case - middle element
        let target = size / 2;
        group.bench_with_input(BenchmarkId::new("binary_best", size), size, |b, _| {
            b.iter(|| binary_search(black_box(&arr), black_box(&target)))
        });
        group.bench_with_input(BenchmarkId::new("linear_best", size), size, |b, _| {
            b.iter(|| linear_search(black_box(&arr), black_box(&target)))
        });

        // Worst case - not found
        let target = *size + 1;
        group.bench_with_input(BenchmarkId::new("binary_worst", size), size, |b, _| {
            b.iter(|| binary_search(black_box(&arr), black_box(&target)))
        });
        group.bench_with_input(BenchmarkId::new("linear_worst", size), size, |b, _| {
            b.iter(|| linear_search(black_box(&arr), black_box(&target)))
        });

        // Random case
        let target = thread_rng().gen_range(0..*size * 2);
        group.bench_with_input(BenchmarkId::new("binary_random", size), size, |b, _| {
            b.iter(|| binary_search(black_box(&arr), black_box(&target)))
        });
        group.bench_with_input(BenchmarkId::new("linear_random", size), size, |b, _| {
            b.iter(|| linear_search(black_box(&arr), black_box(&target)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_search_algorithms);
criterion_main!(benches);