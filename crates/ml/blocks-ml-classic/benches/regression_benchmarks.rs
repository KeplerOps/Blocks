use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::{Array1, Array2};
use blocks_ml_classic::algorithms::regression::linear::LinearRegression;
use blocks_ml_classic::traits::Supervised;

fn bench_linear_regression(c: &mut Criterion) {
    // Generate synthetic data: y = 2x₁ + 3x₂ + 1 + noise
    let n_samples = 10000;
    let x = Array2::from_shape_fn((n_samples, 2), |_| rand::random::<f64>());
    let y = &x.column(0) * 2.0 + &x.column(1) * 3.0 + 1.0 + Array1::from_shape_fn(n_samples, |_| {
        (rand::random::<f64>() - 0.5) * 0.1
    });
    
    let mut group = c.benchmark_group("linear_regression");
    
    // Benchmark fitting
    group.bench_function("fit_10000x2", |b| {
        b.iter(|| {
            let mut model = LinearRegression::new();
            model.fit(black_box(&x), black_box(&y)).unwrap();
        })
    });
    
    // Benchmark prediction
    let mut model = LinearRegression::new();
    model.fit(&x, &y).unwrap();
    let x_test = Array2::from_shape_fn((1000, 2), |_| rand::random::<f64>());
    
    group.bench_function("predict_1000x2", |b| {
        b.iter(|| {
            model.predict(black_box(&x_test)).unwrap();
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_linear_regression);
criterion_main!(benches);