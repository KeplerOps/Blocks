#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_knn_new_valid_config() {
        let config = KNNConfig {
            k: 3,
            metric: DistanceMetric::Euclidean,
            voting: VotingStrategy::Majority,
        };
        assert!(KNN::new(config).is_ok());
    }

    #[test]
    fn test_knn_new_invalid_k() {
        let config = KNNConfig {
            k: 0,
            ..Default::default()
        };
        assert!(matches!(KNN::new(config).unwrap_err(), KNNError::InvalidK));
    }

    #[test]
    fn test_knn_fit_empty_dataset() {
        let mut knn = KNN::new(KNNConfig::default()).unwrap();
        let x = Array2::<f64>::zeros((0, 2));
        let y = Array1::<f64>::zeros(0);
        assert!(matches!(
            knn.fit(x.view(), y.view()),
            Err(KNNError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_knn_fit_labels_mismatch() {
        let mut knn = KNN::new(KNNConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y = arr1(&[1.0]);
        assert!(matches!(
            knn.fit(x.view(), y.view()),
            Err(KNNError::LabelsMismatch)
        ));
    }

    #[test]
    fn test_knn_predict_without_fit() {
        let knn = KNN::new(KNNConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0]]);
        assert!(matches!(
            knn.predict(x.view()),
            Err(KNNError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_knn_predict_dimension_mismatch() {
        let mut knn = KNN::new(KNNConfig::default()).unwrap();
        let x_train = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y_train = arr1(&[0.0, 1.0]);
        knn.fit(x_train.view(), y_train.view()).unwrap();

        let x_test = arr2(&[[1.0], [2.0]]);
        assert!(matches!(
            knn.predict(x_test.view()),
            Err(KNNError::DimensionMismatch)
        ));
    }

    #[test]
    fn test_knn_binary_classification() {
        let mut knn = KNN::new(KNNConfig {
            k: 3,
            metric: DistanceMetric::Euclidean,
            voting: VotingStrategy::Majority,
        })
        .unwrap();

        // Simple binary classification dataset
        let x_train = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [2.0, 2.0],
            [2.0, 3.0],
            [3.0, 2.0],
        ]);
        let y_train = arr1(&[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        knn.fit(x_train.view(), y_train.view()).unwrap();

        // Test points
        let x_test = arr2(&[[0.5, 0.5], [2.5, 2.5]]);
        let predictions = knn.predict(x_test.view()).unwrap();

        assert_eq!(predictions[0], 0.0); // Should be class 0
        assert_eq!(predictions[1], 1.0); // Should be class 1
    }

    #[test]
    fn test_knn_regression() {
        let mut knn = KNN::new(KNNConfig {
            k: 2,
            metric: DistanceMetric::Euclidean,
            voting: VotingStrategy::WeightedDistance,
        })
        .unwrap();

        // Simple regression dataset
        let x_train = arr2(&[[1.0], [2.0], [3.0], [4.0]]);
        let y_train = arr1(&[2.0, 4.0, 6.0, 8.0]); // y = 2x

        knn.fit(x_train.view(), y_train.view()).unwrap();

        // Test points
        let x_test = arr2(&[[1.5], [3.5]]);
        let predictions = knn.predict(x_test.view()).unwrap();

        assert_relative_eq!(predictions[0], 3.0, epsilon = 0.1); // Should be close to 3.0
        assert_relative_eq!(predictions[1], 7.0, epsilon = 0.1); // Should be close to 7.0
    }

    #[test]
    fn test_knn_manhattan_distance() {
        let mut knn = KNN::new(KNNConfig {
            k: 1,
            metric: DistanceMetric::Manhattan,
            voting: VotingStrategy::Majority,
        })
        .unwrap();

        let x_train = arr2(&[[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]]);
        let y_train = arr1(&[0.0, 1.0, 2.0]);

        knn.fit(x_train.view(), y_train.view()).unwrap();

        // Test point at (1, 1) - should be closest to (0, 0) using Manhattan distance
        let x_test = arr2(&[[1.0, 1.0]]);
        let predictions = knn.predict(x_test.view()).unwrap();

        assert_eq!(predictions[0], 0.0);
    }

    #[test]
    fn test_knn_cosine_distance() {
        let mut knn = KNN::new(KNNConfig {
            k: 1,
            metric: DistanceMetric::Cosine,
            voting: VotingStrategy::Majority,
        })
        .unwrap();

        let x_train = arr2(&[
            [1.0, 1.0],   // 45 degrees
            [0.0, 1.0],   // 90 degrees
            [-1.0, 1.0],  // 135 degrees
        ]);
        let y_train = arr1(&[0.0, 1.0, 2.0]);

        knn.fit(x_train.view(), y_train.view()).unwrap();

        // Test point at roughly 60 degrees - should be closest to 45 degrees
        let x_test = arr2(&[[1.0, 1.732]]);  // tan(60) ≈ 1.732
        let predictions = knn.predict(x_test.view()).unwrap();

        assert_eq!(predictions[0], 0.0);
    }

    #[test]
    fn test_knn_minkowski_distance() {
        let mut knn = KNN::new(KNNConfig {
            k: 1,
            metric: DistanceMetric::Minkowski(3.0),
            voting: VotingStrategy::Majority,
        })
        .unwrap();

        let x_train = arr2(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]);
        let y_train = arr1(&[0.0, 1.0, 2.0]);

        knn.fit(x_train.view(), y_train.view()).unwrap();

        let x_test = arr2(&[[0.5, 0.5]]);
        let predictions = knn.predict(x_test.view()).unwrap();

        assert_eq!(predictions[0], 1.0);
    }

    #[test]
    fn test_knn_predict_proba() {
        let mut knn = KNN::new(KNNConfig {
            k: 5,
            metric: DistanceMetric::Euclidean,
            voting: VotingStrategy::WeightedDistance,
        })
        .unwrap();

        // Training data with 3 classes: 0, 1, 2
        let x_train = arr2(&[
            [0.0, 0.0], [0.1, 0.1], [0.2, 0.2],  // Class 0
            [2.0, 2.0], [2.1, 2.1], [2.2, 2.2],  // Class 1
            [4.0, 4.0], [4.1, 4.1], [4.2, 4.2],  // Class 2
        ]);
        let y_train = arr1(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);

        knn.fit(x_train.view(), y_train.view()).unwrap();

        // Test point
        let x_test = arr2(&[[0.1, 0.2]]);  // Should be closest to class 0
        let probabilities = knn.predict_proba(x_test.view()).unwrap();

        assert_eq!(probabilities.shape(), &[1, 3]);  // One row, three classes
        
        // Class 0 should have highest probability
        assert!(probabilities[[0, 0]] > probabilities[[0, 1]]);
        assert!(probabilities[[0, 0]] > probabilities[[0, 2]]);
        
        // Probabilities should sum to approximately 1
        let sum: f64 = probabilities.row(0).sum();
        assert_relative_eq!(sum, 1.0, epsilon = 1e-10);
    }
}