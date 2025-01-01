use blocks_cs_search::algorithms::linear_search;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};

fn bench_linear_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    
    // Small array (best case - first element)
    let small_arr: Vec<i32> = (0..10).collect();
    group.bench_function("small_best", |b| {
        b.iter(|| linear_search(black_box(&small_arr), black_box(&0)))
    });

    // Small array (worst case - not found)
    group.bench_function("small_worst", |b| {
        b.iter(|| linear_search(black_box(&small_arr), black_box(&-1)))
    });

    // Medium array (average case - middle element)
    let medium_arr: Vec<i32> = (0..1000).collect();
    group.bench_function("medium_avg", |b| {
        b.iter(|| linear_search(black_box(&medium_arr), black_box(&500)))
    });

    // Large array (random position)
    let large_arr: Vec<i32> = (0..100_000).collect();
    let random_target = thread_rng().gen_range(0..100_000);
    group.bench_function("large_random", |b| {
        b.iter(|| linear_search(black_box(&large_arr), black_box(&random_target)))
    });

    group.finish();
}

criterion_group!(benches, bench_linear_search);
criterion_main!(benches);