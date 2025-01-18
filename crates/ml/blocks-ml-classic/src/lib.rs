/*!
This crate provides implementations of classic machine learning algorithms in Rust.

Each algorithm is implemented with a focus on:
- Type safety and compile-time guarantees
- Performance optimizations
- Memory efficiency
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Clustering
- [`KMeans`](algorithms::clustering::kmeans): k-Means clustering algorithm
- [`KNN`](algorithms::clustering::knn): k-Nearest Neighbors algorithm

## Regression
- [`LinearRegression`](algorithms::regression::linear): Ordinary Least Squares (OLS) regression
- [`LogisticRegression`](algorithms::regression::logistic): Binary and multiclass logistic regression

## Tree-based Methods
- [`DecisionTree`](algorithms::trees::decision_tree): ID3 and C4.5 decision tree algorithms
- [`RandomForest`](algorithms::trees::random_forest): Random Forest ensemble method

## Other Algorithms
- [`SVM`](algorithms::svm): Support Vector Machine implementation
- [`NaiveBayes`](algorithms::naive_bayes): Naive Bayes classifier
- [`GradientBoosting`](algorithms::boosting::gbm): Gradient Boosting Machines
- [`XGBoost`](algorithms::boosting::xgboost): XGBoost implementation

# Usage Examples

## K-Means Clustering

```rust,no_run
use blocks_ml_classic::algorithms::clustering::kmeans::KMeans;
use blocks_ml_classic::traits::Unsupervised;
use ndarray::Array2;

// Create a dataset
let data = Array2::from_shape_vec((100, 2), vec![/* ... */]).unwrap();

// Initialize and fit k-means
let mut kmeans = KMeans::new(3); // 3 clusters
kmeans.fit(&data).expect("Failed to fit k-means");

// Transform data to get distances to centroids
let distances = kmeans.transform(&data).expect("Failed to transform data");
```

## K-Nearest Neighbors

```rust,no_run
use blocks_ml_classic::algorithms::clustering::knn::KNN;
use blocks_ml_classic::traits::Supervised;
use ndarray::{Array1, Array2};

// Create training data
let x_train = Array2::from_shape_vec((100, 2), vec![/* ... */]).unwrap();
let y_train = Array1::from_shape_vec(100, vec![/* ... */]).unwrap();

// Initialize and fit k-NN
let mut knn = KNN::new(5); // 5 neighbors
knn.fit(&x_train, &y_train).expect("Failed to fit k-NN");

// Make predictions
let x_test = Array2::from_shape_vec((10, 2), vec![/* ... */]).unwrap();
let predictions = knn.predict(&x_test).expect("Failed to predict");
```

# Features
- `parallel`: Enable parallel processing using rayon
- `serde-support`: Enable serialization/deserialization support
*/

pub mod algorithms;
pub mod error;
pub mod traits;
mod utils;

pub use error::{Error, Result};
