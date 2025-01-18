use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::Array2;
use blocks_ml_classic::algorithms::clustering::kmeans::KMeans;

fn bench_kmeans(c: &mut Criterion) {
    let data = Array2::from_shape_fn((1000, 10), |_| rand::random::<f64>());
    
    let mut group = c.benchmark_group("kmeans");
    group.bench_function("fit_1000x10", |b| {
        b.iter(|| {
            let mut kmeans = KMeans::new(black_box(3));
            kmeans.fit(black_box(&data)).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_kmeans);
criterion_main!(benches);