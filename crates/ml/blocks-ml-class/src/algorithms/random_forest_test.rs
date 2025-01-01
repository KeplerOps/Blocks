#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_random_forest_new() {
        let config = RandomForestConfig::default();
        let forest = RandomForest::new(config).unwrap();
        assert_eq!(forest.n_trees(), 0);
        assert!(forest.feature_importances().is_none());
        assert!(forest.oob_score().is_none());
    }

    #[test]
    fn test_random_forest_new_invalid_config() {
        // Test invalid tree count
        let config = RandomForestConfig {
            n_trees: 0,
            ..Default::default()
        };
        assert!(matches!(
            RandomForest::new(config),
            Err(RandomForestError::InvalidTreeCount)
        ));

        // Test invalid bootstrap ratio
        let config = RandomForestConfig {
            bootstrap_ratio: 0.0,
            ..Default::default()
        };
        assert!(matches!(
            RandomForest::new(config),
            Err(RandomForestError::InvalidBootstrapRatio)
        ));

        let config = RandomForestConfig {
            bootstrap_ratio: 1.1,
            ..Default::default()
        };
        assert!(matches!(
            RandomForest::new(config),
            Err(RandomForestError::InvalidBootstrapRatio)
        ));
    }

    #[test]
    fn test_random_forest_fit_empty_dataset() {
        let mut forest = RandomForest::new(RandomForestConfig::default()).unwrap();
        let x = Array2::<f64>::zeros((0, 2));
        let y = Array1::<f64>::zeros(0);
        assert!(matches!(
            forest.fit(x.view(), y.view()),
            Err(RandomForestError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_random_forest_fit_labels_mismatch() {
        let mut forest = RandomForest::new(RandomForestConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y = arr1(&[1.0]);
        assert!(matches!(
            forest.fit(x.view(), y.view()),
            Err(RandomForestError::LabelsMismatch)
        ));
    }

    #[test]
    fn test_random_forest_predict_without_fit() {
        let forest = RandomForest::new(RandomForestConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0]]);
        assert!(matches!(
            forest.predict(x.view()),
            Err(RandomForestError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_random_forest_predict_dimension_mismatch() {
        let mut forest = RandomForest::new(RandomForestConfig::default()).unwrap();
        let x_train = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y_train = arr1(&[0.0, 1.0]);
        forest.fit(x_train.view(), y_train.view()).unwrap();

        let x_test = arr2(&[[1.0], [2.0]]);
        assert!(matches!(
            forest.predict(x_test.view()),
            Err(RandomForestError::DimensionMismatch)
        ));
    }

    #[test]
    fn test_random_forest_binary_classification() {
        let mut forest = RandomForest::new(RandomForestConfig {
            n_trees: 10,
            bootstrap_ratio: 0.7,
            ..Default::default()
        })
        .unwrap();

        // Simple binary classification dataset
        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.1, 0.1],
            [0.9, 0.9],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0, 0.0, 1.0]);

        forest.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = forest.predict(x.view()).unwrap();
        assert_eq!(predictions.len(), x.nrows());
        assert!(predictions.iter().all(|&p| p == 0.0 || p == 1.0));

        // Test probabilities
        let probas = forest.predict_proba(x.view()).unwrap();
        assert_eq!(probas.shape(), &[x.nrows(), 2]);
        assert!(probas.iter().all(|&p| p >= 0.0 && p <= 1.0));

        // Each row should sum to approximately 1
        for row in probas.rows() {
            assert_relative_eq!(row.sum(), 1.0, epsilon = 1e-10);
        }

        // Test feature importances
        let importances = forest.feature_importances().unwrap();
        assert_eq!(importances.len(), 2);
        assert!(importances.iter().all(|&x| x >= 0.0 && x <= 1.0));
        assert_relative_eq!(importances.sum(), 1.0, epsilon = 1e-10);

        // Test OOB score
        assert!(forest.oob_score().is_some());
        assert!(forest.oob_score().unwrap() >= 0.0 && forest.oob_score().unwrap() <= 1.0);
    }

    #[test]
    fn test_random_forest_multiclass() {
        let mut forest = RandomForest::new(RandomForestConfig {
            n_trees: 10,
            bootstrap_ratio: 0.7,
            ..Default::default()
        })
        .unwrap();

        // Multiclass dataset
        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.5, 0.5],
            [0.5, 0.6],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);

        forest.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = forest.predict(x.view()).unwrap();
        assert_eq!(predictions.len(), x.nrows());
        assert!(predictions.iter().all(|&p| p == 0.0 || p == 1.0 || p == 2.0));

        // Test probabilities
        let probas = forest.predict_proba(x.view()).unwrap();
        assert_eq!(probas.shape(), &[x.nrows(), 3]);
        assert!(probas.iter().all(|&p| p >= 0.0 && p <= 1.0));

        // Each row should sum to approximately 1
        for row in probas.rows() {
            assert_relative_eq!(row.sum(), 1.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_random_forest_parallel_training() {
        let mut forest_single = RandomForest::new(RandomForestConfig {
            n_trees: 10,
            n_jobs: Some(1),
            ..Default::default()
        })
        .unwrap();

        let mut forest_parallel = RandomForest::new(RandomForestConfig {
            n_trees: 10,
            n_jobs: Some(4),
            ..Default::default()
        })
        .unwrap();

        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        // Both should train successfully
        forest_single.fit(x.view(), y.view()).unwrap();
        forest_parallel.fit(x.view(), y.view()).unwrap();

        assert_eq!(forest_single.n_trees(), forest_parallel.n_trees());
    }

    #[test]
    fn test_random_forest_bootstrap_sampling() {
        let mut forest = RandomForest::new(RandomForestConfig {
            n_trees: 5,
            bootstrap_ratio: 0.5,  // Small ratio to ensure different samples
            ..Default::default()
        })
        .unwrap();

        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.5, 0.5],
            [0.5, 0.6],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);

        forest.fit(x.view(), y.view()).unwrap();

        // Each tree should have seen different samples
        assert!(forest.oob_score().is_some());
    }

    #[test]
    fn test_random_forest_feature_importance_consistency() {
        let mut forest = RandomForest::new(RandomForestConfig {
            n_trees: 10,
            ..Default::default()
        })
        .unwrap();

        // Dataset where first feature is more important
        let x = arr2(&[
            [0.0, 0.5],
            [0.1, 0.4],
            [0.9, 0.6],
            [1.0, 0.5],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        forest.fit(x.view(), y.view()).unwrap();

        let importances = forest.feature_importances().unwrap();
        assert!(importances[0] > importances[1]); // First feature should be more important
    }
}