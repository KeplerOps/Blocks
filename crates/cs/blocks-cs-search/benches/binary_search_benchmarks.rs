use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use blocks_cs_search::binary_search::binary_search;
mod common;
use common::{generate_data, SIZES, DISTRIBUTIONS};

fn benchmark_binary_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_search");
    group.sample_size(10);

    for &size in &SIZES {
        for dist in &DISTRIBUTIONS {
            let bench_name = format!("binary_search_{}_{}", dist, size);
            group.bench_function(&bench_name, |b| {
                b.iter_batched(
                    || generate_data(size, dist),
                    |data| {
                        let target = data[size / 2];
                        binary_search(&data, target)
                    },
                    BatchSize::LargeInput,
                )
            });
        }
    }

    group.finish();
}

criterion_group!(benches, benchmark_binary_search);
criterion_main!(benches);
