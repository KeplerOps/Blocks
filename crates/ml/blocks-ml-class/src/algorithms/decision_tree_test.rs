#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_decision_tree_new() {
        let config = DecisionTreeConfig::default();
        let tree = DecisionTree::new(config).unwrap();
        assert!(tree.feature_importances().is_none());
        assert_eq!(tree.depth(), 0);
        assert_eq!(tree.node_count(), 0);
        assert_eq!(tree.leaf_count(), 0);
    }

    #[test]
    fn test_decision_tree_new_invalid_config() {
        // Test invalid min_samples_split
        let config = DecisionTreeConfig {
            min_samples_split: 1,
            ..Default::default()
        };
        assert!(matches!(
            DecisionTree::new(config),
            Err(DecisionTreeError::InvalidPruningParams)
        ));

        // Test invalid min_samples_leaf
        let config = DecisionTreeConfig {
            min_samples_leaf: 0,
            ..Default::default()
        };
        assert!(matches!(
            DecisionTree::new(config),
            Err(DecisionTreeError::InvalidPruningParams)
        ));

        // Test invalid min_impurity_decrease
        let config = DecisionTreeConfig {
            min_impurity_decrease: -1.0,
            ..Default::default()
        };
        assert!(matches!(
            DecisionTree::new(config),
            Err(DecisionTreeError::InvalidPruningParams)
        ));
    }

    #[test]
    fn test_decision_tree_fit_empty_dataset() {
        let mut tree = DecisionTree::new(DecisionTreeConfig::default()).unwrap();
        let x = Array2::<f64>::zeros((0, 2));
        let y = Array1::<f64>::zeros(0);
        assert!(matches!(
            tree.fit(x.view(), y.view()),
            Err(DecisionTreeError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_decision_tree_fit_labels_mismatch() {
        let mut tree = DecisionTree::new(DecisionTreeConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y = arr1(&[1.0]);
        assert!(matches!(
            tree.fit(x.view(), y.view()),
            Err(DecisionTreeError::LabelsMismatch)
        ));
    }

    #[test]
    fn test_decision_tree_predict_without_fit() {
        let tree = DecisionTree::new(DecisionTreeConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0]]);
        assert!(matches!(
            tree.predict(x.view()),
            Err(DecisionTreeError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_decision_tree_predict_dimension_mismatch() {
        let mut tree = DecisionTree::new(DecisionTreeConfig::default()).unwrap();
        let x_train = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y_train = arr1(&[0.0, 1.0]);
        tree.fit(x_train.view(), y_train.view()).unwrap();

        let x_test = arr2(&[[1.0], [2.0]]);
        assert!(matches!(
            tree.predict(x_test.view()),
            Err(DecisionTreeError::DimensionMismatch)
        ));
    }

    #[test]
    fn test_decision_tree_perfect_split() {
        let mut tree = DecisionTree::new(DecisionTreeConfig {
            max_depth: Some(2),
            min_samples_split: 2,
            min_samples_leaf: 1,
            criterion: SplitCriterion::InformationGain,
            max_features: None,
            min_impurity_decrease: 0.0,
        })
        .unwrap();

        // Perfect split dataset
        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        tree.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = tree.predict(x.view()).unwrap();
        assert_eq!(predictions, y);

        // Test tree structure
        assert!(tree.depth() <= 2);
        assert!(tree.node_count() > 1);
        assert!(tree.leaf_count() >= 2);

        // Test feature importances
        let importances = tree.feature_importances().unwrap();
        assert_eq!(importances.len(), 2);
        assert!(importances.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }

    #[test]
    fn test_decision_tree_continuous_features() {
        let mut tree = DecisionTree::new(DecisionTreeConfig {
            max_depth: Some(3),
            min_samples_split: 2,
            min_samples_leaf: 1,
            criterion: SplitCriterion::GainRatio,
            max_features: None,
            min_impurity_decrease: 0.0,
        })
        .unwrap();

        // Continuous feature dataset
        let x = arr2(&[
            [0.1, 0.2],
            [0.2, 0.3],
            [0.8, 0.7],
            [0.9, 0.8],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        tree.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = tree.predict(x.view()).unwrap();
        assert_eq!(predictions, y);

        // Test on new points
        let x_test = arr2(&[
            [0.15, 0.25],  // Should be class 0
            [0.85, 0.75],  // Should be class 1
        ]);
        let predictions = tree.predict(x_test.view()).unwrap();
        assert_eq!(predictions, arr1(&[0.0, 1.0]));
    }

    #[test]
    fn test_decision_tree_max_features() {
        let mut tree = DecisionTree::new(DecisionTreeConfig {
            max_features: Some(1),
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

        tree.fit(x.view(), y.view()).unwrap();

        // Only one feature should have non-zero importance
        let importances = tree.feature_importances().unwrap();
        assert_eq!(importances.iter().filter(|&&x| x > 0.0).count(), 1);
    }

    #[test]
    fn test_decision_tree_min_impurity_decrease() {
        let mut tree_with_threshold = DecisionTree::new(DecisionTreeConfig {
            min_impurity_decrease: 0.5,  // High threshold
            ..Default::default()
        })
        .unwrap();

        let mut tree_without_threshold = DecisionTree::new(DecisionTreeConfig {
            min_impurity_decrease: 0.0,
            ..Default::default()
        })
        .unwrap();

        let x = arr2(&[
            [0.1, 0.1],
            [0.2, 0.2],
            [0.3, 0.3],
            [0.8, 0.8],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        tree_with_threshold.fit(x.view(), y.view()).unwrap();
        tree_without_threshold.fit(x.view(), y.view()).unwrap();

        // Tree with threshold should be smaller
        assert!(tree_with_threshold.node_count() < tree_without_threshold.node_count());
    }

    #[test]
    fn test_decision_tree_min_samples() {
        let mut tree = DecisionTree::new(DecisionTreeConfig {
            min_samples_split: 3,
            min_samples_leaf: 2,
            ..Default::default()
        })
        .unwrap();

        let x = arr2(&[
            [0.0, 0.0],
            [0.1, 0.1],
            [0.2, 0.2],
            [0.8, 0.8],
            [0.9, 0.9],
        ]);
        let y = arr1(&[0.0, 0.0, 0.0, 1.0, 1.0]);

        tree.fit(x.view(), y.view()).unwrap();

        // Each leaf should have at least 2 samples
        assert!(tree.leaf_count() <= 2);
    }

    #[test]
    fn test_decision_tree_multiclass() {
        let mut tree = DecisionTree::new(DecisionTreeConfig::default()).unwrap();

        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.5, 0.5],
            [0.5, 0.6],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);

        tree.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = tree.predict(x.view()).unwrap();
        assert_eq!(predictions, y);

        // Test feature importances
        let importances = tree.feature_importances().unwrap();
        assert_eq!(importances.len(), 2);
        assert!(importances.iter().sum::<f64>() > 0.0);
    }
}