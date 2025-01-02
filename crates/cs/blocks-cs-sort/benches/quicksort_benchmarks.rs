use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use rand::prelude::*;
use blocks_cs_sort::algorithms::quicksort;

/// Helper to generate different input distributions.
fn generate_data(len: usize, distribution: &str) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut data: Vec<i32> = (0..len as i32).collect();

    match distribution {
        "sorted" => { /* already sorted */ },
        "reverse" => data.reverse(),
        "random" => data.shuffle(&mut rng),
        "nearly_sorted" => {
            // Swap every 100th element
            for i in 0..(len/100) {
                let j = i * 100;
                if j + 1 < len {
                    data.swap(j, j + 1);
                }
            }
        },
        "few_unique" => {
            // Only use values 0-9 repeated
            for i in 0..len {
                data[i] = (i % 10) as i32;
            }
        },
        _ => {}
    };
    data
}

fn benchmark_quicksort(c: &mut Criterion) {
    let sizes = [1_000, 10_000, 100_000, 1_000_000];
    let distributions = ["sorted", "reverse", "random", "nearly_sorted", "few_unique"];

    let mut group = c.benchmark_group("quicksort");
    group.sample_size(10); // Adjust based on your needs

    for &size in &sizes {
        for dist in &distributions {
            let bench_name = format!("quicksort_{}_{}", dist, size);

            group.bench_function(&bench_name, |b| {
                b.iter_batched(
                    || generate_data(size, dist),
                    |mut data| quicksort::sort(&mut data),
                    BatchSize::LargeInput,
                )
            });
        }
    }

    group.finish();
}

#[cfg(feature = "parallel")]
fn benchmark_parallel_quicksort(c: &mut Criterion) {
    let sizes = [100_000, 1_000_000, 10_000_000];
    let distributions = ["sorted", "reverse", "random", "nearly_sorted", "few_unique"];

    let mut group = c.benchmark_group("parallel_quicksort");
    group.sample_size(10); // Fewer samples for large parallel sorts

    for &size in &sizes {
        for dist in &distributions {
            let bench_name = format!("parallel_quicksort_{}_{}", dist, size);

            group.bench_function(&bench_name, |b| {
                b.iter_batched(
                    || generate_data(size, dist),
                    |mut data| quicksort::sort(&mut data),
                    BatchSize::LargeInput,
                )
            });
        }
    }

    group.finish();
}

fn benchmark_std_sort(c: &mut Criterion) {
    let sizes = [1_000, 10_000, 100_000, 1_000_000];
    let distributions = ["sorted", "reverse", "random", "nearly_sorted", "few_unique"];

    let mut group = c.benchmark_group("std_sort");
    group.sample_size(10);

    for &size in &sizes {
        for dist in &distributions {
            let bench_name = format!("std_sort_{}_{}", dist, size);

            group.bench_function(&bench_name, |b| {
                b.iter_batched(
                    || generate_data(size, dist),
                    |mut data| data.sort(),
                    BatchSize::LargeInput,
                )
            });
        }
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn benchmark_simd_sort(c: &mut Criterion) {
    let sizes = [1_000, 10_000, 100_000, 1_000_000];
    let distributions = ["sorted", "reverse", "random", "nearly_sorted", "few_unique"];

    let mut group = c.benchmark_group("simd_sort");
    group.sample_size(10);

    for &size in &sizes {
        for dist in &distributions {
            let bench_name = format!("simd_sort_{}_{}", dist, size);

            group.bench_function(&bench_name, |b| {
                b.iter_batched(
                    || generate_data(size, dist),
                    |mut data| quicksort::sort_i32(&mut data).expect("Sort should succeed"),
                    BatchSize::LargeInput,
                )
            });
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_quicksort, benchmark_std_sort
}

#[cfg(feature = "parallel")]
criterion_group! {
    name = parallel_benches;
    config = Criterion::default();
    targets = benchmark_parallel_quicksort
}

#[cfg(feature = "simd")]
criterion_group! {
    name = simd_benches;
    config = Criterion::default();
    targets = benchmark_simd_sort
}

// Base benchmarks always run
#[cfg(not(any(feature = "parallel", feature = "simd")))]
criterion_main!(benches);

// With parallel feature only
#[cfg(all(feature = "parallel", not(feature = "simd")))]
criterion_main!(benches, parallel_benches);

// With SIMD feature only
#[cfg(all(feature = "simd", not(feature = "parallel")))]
criterion_main!(benches, simd_benches);

// With both parallel and SIMD features
#[cfg(all(feature = "parallel", feature = "simd"))]
criterion_main!(benches, parallel_benches, simd_benches);