use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use num_traits::Float;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum KNNError {
    #[error("k must be greater than 0")]
    InvalidK,
    #[error("Empty training dataset")]
    EmptyTrainingSet,
    #[error("Empty test dataset")]
    EmptyTestSet,
    #[error("Feature dimensions mismatch")]
    DimensionMismatch,
    #[error("No labels provided for training data")]
    NoLabels,
    #[error("Labels length mismatch with training data")]
    LabelsMismatch,
}

/// Distance metrics supported by KNN
#[derive(Debug, Clone, Copy)]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
    Cosine,
    Minkowski(f64), // p-norm parameter
}

/// Voting strategy for classification
#[derive(Debug, Clone, Copy)]
pub enum VotingStrategy {
    Majority,           // Simple majority voting
    WeightedDistance,   // Weight votes by inverse distance
}

/// Configuration for KNN algorithm
#[derive(Debug, Clone)]
pub struct KNNConfig {
    /// Number of neighbors to consider
    pub k: usize,
    /// Distance metric to use
    pub metric: DistanceMetric,
    /// Voting strategy for classification
    pub voting: VotingStrategy,
}

impl Default for KNNConfig {
    fn default() -> Self {
        Self {
            k: 5,
            metric: DistanceMetric::Euclidean,
            voting: VotingStrategy::Majority,
        }
    }
}

/// K-Nearest Neighbors implementation supporting both classification and regression
#[derive(Debug)]
pub struct KNN {
    config: KNNConfig,
    x_train: Option<Array2<f64>>,
    y_train: Option<Array1<f64>>,
}

impl KNN {
    /// Creates a new KNN instance with the given configuration
    pub fn new(config: KNNConfig) -> Result<Self, KNNError> {
        if config.k == 0 {
            return Err(KNNError::InvalidK);
        }

        Ok(Self {
            config,
            x_train: None,
            y_train: None,
        })
    }

    /// Fits the KNN model with training data
    pub fn fit(
        &mut self,
        x: ArrayView2<f64>,
        y: ArrayView1<f64>,
    ) -> Result<(), KNNError> {
        if x.nrows() == 0 {
            return Err(KNNError::EmptyTrainingSet);
        }
        if y.len() != x.nrows() {
            return Err(KNNError::LabelsMismatch);
        }

        self.x_train = Some(x.to_owned());
        self.y_train = Some(y.to_owned());
        Ok(())
    }

    /// Predicts labels for new data points
    pub fn predict(&self, x: ArrayView2<f64>) -> Result<Array1<f64>, KNNError> {
        let x_train = self.x_train.as_ref().ok_or(KNNError::EmptyTrainingSet)?;
        let y_train = self.y_train.as_ref().ok_or(KNNError::EmptyTrainingSet)?;

        if x.ncols() != x_train.ncols() {
            return Err(KNNError::DimensionMismatch);
        }

        let mut predictions = Array1::zeros(x.nrows());
        for (i, point) in x.outer_iter().enumerate() {
            let mut distances: Vec<(f64, f64)> = Vec::with_capacity(x_train.nrows());
            
            // Calculate distances to all training points
            for (j, train_point) in x_train.outer_iter().enumerate() {
                let dist = match self.config.metric {
                    DistanceMetric::Euclidean => euclidean_distance(point.view(), train_point.view()),
                    DistanceMetric::Manhattan => manhattan_distance(point.view(), train_point.view()),
                    DistanceMetric::Cosine => cosine_distance(point.view(), train_point.view()),
                    DistanceMetric::Minkowski(p) => minkowski_distance(point.view(), train_point.view(), p),
                };
                distances.push((dist, y_train[j]));
            }

            // Sort by distance
            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            
            // Take k nearest neighbors
            let k_nearest = &distances[..self.config.k.min(distances.len())];

            match self.config.voting {
                VotingStrategy::Majority => {
                    // Simple majority voting
                    let mut class_counts: HashMap<f64, f64> = HashMap::new();
                    for &(_, label) in k_nearest {
                        *class_counts.entry(label).or_default() += 1.0;
                    }
                    predictions[i] = class_counts
                        .into_iter()
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        .unwrap()
                        .0;
                }
                VotingStrategy::WeightedDistance => {
                    // Weight votes by inverse distance
                    let mut weighted_sum = 0.0;
                    let mut weight_sum = 0.0;
                    for &(dist, label) in k_nearest {
                        let weight = 1.0 / (dist + f64::EPSILON);
                        weighted_sum += weight * label;
                        weight_sum += weight;
                    }
                    predictions[i] = weighted_sum / weight_sum;
                }
            }
        }

        Ok(predictions)
    }

    /// Predicts probabilities for each class (classification only)
    pub fn predict_proba(&self, x: ArrayView2<f64>) -> Result<Array2<f64>, KNNError> {
        let x_train = self.x_train.as_ref().ok_or(KNNError::EmptyTrainingSet)?;
        let y_train = self.y_train.as_ref().ok_or(KNNError::EmptyTrainingSet)?;

        if x.ncols() != x_train.ncols() {
            return Err(KNNError::DimensionMismatch);
        }

        // Find unique classes
        let mut unique_classes: Vec<f64> = y_train.iter().copied().collect();
        unique_classes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_classes.dedup();
        let n_classes = unique_classes.len();

        let mut probabilities = Array2::zeros((x.nrows(), n_classes));

        for (i, point) in x.outer_iter().enumerate() {
            let mut distances: Vec<(f64, f64)> = Vec::with_capacity(x_train.nrows());
            
            // Calculate distances to all training points
            for (j, train_point) in x_train.outer_iter().enumerate() {
                let dist = match self.config.metric {
                    DistanceMetric::Euclidean => euclidean_distance(point.view(), train_point.view()),
                    DistanceMetric::Manhattan => manhattan_distance(point.view(), train_point.view()),
                    DistanceMetric::Cosine => cosine_distance(point.view(), train_point.view()),
                    DistanceMetric::Minkowski(p) => minkowski_distance(point.view(), train_point.view(), p),
                };
                distances.push((dist, y_train[j]));
            }

            // Sort by distance
            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            
            // Take k nearest neighbors
            let k_nearest = &distances[..self.config.k.min(distances.len())];

            match self.config.voting {
                VotingStrategy::Majority => {
                    // Count occurrences of each class
                    let mut class_counts = vec![0.0; n_classes];
                    for &(_, label) in k_nearest {
                        let idx = unique_classes.binary_search_by(|x| x.partial_cmp(&label).unwrap()).unwrap();
                        class_counts[idx] += 1.0;
                    }
                    // Normalize to probabilities
                    let total: f64 = class_counts.iter().sum();
                    for (j, &count) in class_counts.iter().enumerate() {
                        probabilities[[i, j]] = count / total;
                    }
                }
                VotingStrategy::WeightedDistance => {
                    // Weight by inverse distance
                    let mut class_weights = vec![0.0; n_classes];
                    let mut total_weight = 0.0;
                    for &(dist, label) in k_nearest {
                        let weight = 1.0 / (dist + f64::EPSILON);
                        let idx = unique_classes.binary_search_by(|x| x.partial_cmp(&label).unwrap()).unwrap();
                        class_weights[idx] += weight;
                        total_weight += weight;
                    }
                    // Normalize to probabilities
                    for (j, &weight) in class_weights.iter().enumerate() {
                        probabilities[[i, j]] = weight / total_weight;
                    }
                }
            }
        }

        Ok(probabilities)
    }
}

// Distance metric implementations
fn euclidean_distance(a: ArrayView1<f64>, b: ArrayView1<f64>) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn manhattan_distance(a: ArrayView1<f64>, b: ArrayView1<f64>) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum()
}

fn cosine_distance(a: ArrayView1<f64>, b: ArrayView1<f64>) -> f64 {
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        1.0 // Maximum distance for zero vectors
    } else {
        1.0 - (dot_product / (norm_a * norm_b))
    }
}

fn minkowski_distance(a: ArrayView1<f64>, b: ArrayView1<f64>, p: f64) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs().powf(p))
        .sum::<f64>()
        .powf(1.0 / p)
}
}