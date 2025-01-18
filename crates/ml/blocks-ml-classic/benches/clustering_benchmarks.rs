use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::{Array1, Array2};
use blocks_ml_classic::algorithms::clustering::{kmeans::KMeans, knn::KNN};

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

fn bench_knn(c: &mut Criterion) {
    let x_train = Array2::from_shape_fn((1000, 10), |_| rand::random::<f64>());
    let y_train = Array1::from_shape_fn(1000, |_| (rand::random::<f64>() * 3.0).floor());
    let x_test = Array2::from_shape_fn((100, 10), |_| rand::random::<f64>());
    
    let mut knn = KNN::new(5);
    knn.fit(&x_train, &y_train).unwrap();
    
    let mut group = c.benchmark_group("knn");
    group.bench_function("predict_100x10", |b| {
        b.iter(|| {
            knn.predict(black_box(&x_test)).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_kmeans, bench_knn);
criterion_main!(benches);