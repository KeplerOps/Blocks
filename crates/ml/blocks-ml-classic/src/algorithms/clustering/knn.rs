use ndarray::{Array1, Array2};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::traits::Supervised;

/// k-Nearest Neighbors algorithm implementation
#[derive(Debug)]
pub struct KNN {
    n_neighbors: usize,
    x_train: Option<Array2<f64>>,
    y_train: Option<Array1<f64>>,
}

impl Default for KNN {
    fn default() -> Self {
        Self {
            n_neighbors: 5,
            x_train: None,
            y_train: None,
        }
    }
}

impl KNN {
    /// Create a new KNN instance with the specified number of neighbors
    pub fn new(n_neighbors: usize) -> Self {
        Self {
            n_neighbors,
            ..Default::default()
        }
    }

    /// Get the number of neighbors used for prediction
    pub fn n_neighbors(&self) -> usize {
        self.n_neighbors
    }

    /// Calculate distances between a point and all training points
    fn calculate_distances(&self, x: &Array2<f64>) -> Result<Array2<f64>> {
        let x_train = self.x_train.as_ref().ok_or_else(|| {
            Error::InvalidState("Model must be fitted before prediction".to_string())
        })?;

        let mut distances = Array2::zeros((x.nrows(), x_train.nrows()));

        for (i, sample) in x.rows().into_iter().enumerate() {
            for (j, train_sample) in x_train.rows().into_iter().enumerate() {
                let dist = sample
                    .iter()
                    .zip(train_sample.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                distances[[i, j]] = dist;
            }
        }

        Ok(distances)
    }

    /// Find indices of k nearest neighbors
    fn get_nearest_neighbors(&self, distances: &Array1<f64>) -> Vec<(usize, f64)> {
        let mut neighbors: Vec<_> = distances
            .iter()
            .enumerate()
            .map(|(i, &d)| (i, d))
            .collect();
        
        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        neighbors.truncate(self.n_neighbors);
        
        neighbors
    }
}

impl Supervised for KNN {
    fn fit(&mut self, x: &Array2<f64>, y: &Array1<f64>) -> Result<()> {
        if x.is_empty() {
            return Err(Error::InvalidParameter("Empty input array".to_string()));
        }

        if x.nrows() != y.len() {
            return Err(Error::InvalidDimensions {
                expected: vec![x.nrows()],
                got: vec![y.len()],
            });
        }

        if self.n_neighbors > x.nrows() {
            return Err(Error::InvalidParameter(
                "n_neighbors cannot be greater than number of samples".to_string(),
            ));
        }

        self.x_train = Some(x.clone());
        self.y_train = Some(y.clone());
        Ok(())
    }

    fn predict(&self, x: &Array2<f64>) -> Result<Array1<f64>> {
        let y_train = self.y_train.as_ref().ok_or_else(|| {
            Error::InvalidState("Model must be fitted before prediction".to_string())
        })?;

        let distances = self.calculate_distances(x)?;
        let mut predictions = Array1::zeros(x.nrows());

        for (i, row) in distances.rows().into_iter().enumerate() {
            let neighbors = self.get_nearest_neighbors(&row.to_owned());
            
            // Majority vote for classification
            // Group by label with tolerance for floating-point comparison
            let mut votes: HashMap<i64, usize> = HashMap::new();
            for (idx, _) in neighbors {
                let label = (y_train[idx] * 1e6).round() as i64; // Convert to integer with 6 decimal precision
                *votes.entry(label).or_insert(0) += 1;
            }

            let prediction = votes
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(label, _)| label as f64 / 1e6)
                .unwrap();

            predictions[i] = prediction;
        }

        Ok(predictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knn_basic() {
        // Create a simple dataset with two classes
        let x_train = Array2::from_shape_vec(
            (6, 2),
            vec![
                0.0, 0.0,  // Class 0
                0.1, 0.1,  // Class 0
                0.2, 0.2,  // Class 0
                3.0, 3.0,  // Class 1
                3.1, 3.1,  // Class 1
                3.2, 3.2,  // Class 1
            ],
        )
        .unwrap();

        let y_train = Array1::from_vec(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        let mut knn = KNN::new(3);
        knn.fit(&x_train, &y_train).unwrap();

        // Test predictions
        let x_test = Array2::from_shape_vec(
            (2, 2),
            vec![
                0.15, 0.15,  // Should predict class 0
                3.15, 3.15,  // Should predict class 1
            ],
        )
        .unwrap();

        let predictions = knn.predict(&x_test).unwrap();
        assert_eq!(predictions[0], 0.0);
        assert_eq!(predictions[1], 1.0);
    }

    #[test]
    fn test_knn_empty_input() {
        let x = Array2::zeros((0, 2));
        let y = Array1::zeros(0);
        let mut knn = KNN::new(3);
        assert!(knn.fit(&x, &y).is_err());
    }

    #[test]
    fn test_knn_dimension_mismatch() {
        let x = Array2::zeros((10, 2));
        let y = Array1::zeros(5);
        let mut knn = KNN::new(3);
        assert!(knn.fit(&x, &y).is_err());
    }

    #[test]
    fn test_knn_too_many_neighbors() {
        let x = Array2::zeros((5, 2));
        let y = Array1::zeros(5);
        let mut knn = KNN::new(6);
        assert!(knn.fit(&x, &y).is_err());
    }

    #[test]
    fn test_knn_predict_without_fit() {
        let x = Array2::zeros((3, 2));
        let knn = KNN::new(3);
        assert!(knn.predict(&x).is_err());
    }

    #[test]
    fn test_knn_edge_case() {
        // Test with a single point in each class
        let x_train = Array2::from_shape_vec(
            (2, 2),
            vec![
                0.0, 0.0,  // Class 0
                1.0, 1.0,  // Class 1
            ],
        )
        .unwrap();

        let y_train = Array1::from_vec(vec![0.0, 1.0]);

        let mut knn = KNN::new(1);
        knn.fit(&x_train, &y_train).unwrap();

        // Test point exactly between the two training points
        let x_test = Array2::from_shape_vec(
            (1, 2),
            vec![0.5, 0.5],
        )
        .unwrap();

        let predictions = knn.predict(&x_test).unwrap();
        assert!(predictions[0] == 0.0 || predictions[0] == 1.0); // Either class is valid
    }
}