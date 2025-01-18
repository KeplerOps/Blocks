pub mod clustering;
pub mod regression;
pub mod trees;
pub mod boosting;

// Re-export implemented algorithms
pub use clustering::{kmeans::KMeans, knn::KNN};
pub use regression::{linear::LinearRegression, logistic::LogisticRegression};