#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_logistic_regression_new() {
        let config = LogisticRegressionConfig::default();
        let model = LogisticRegression::new(config).unwrap();
        assert!(model.coefficients().is_none());
        assert!(model.intercepts().is_none());
        assert!(model.classes().is_none());
    }

    #[test]
    fn test_logistic_regression_new_invalid_config() {
        // Test negative regularization
        let config = LogisticRegressionConfig {
            l2_reg: -1.0,
            ..Default::default()
        };
        assert!(matches!(
            LogisticRegression::new(config),
            Err(LogisticRegressionError::InvalidRegularization)
        ));

        // Test invalid learning rate
        let config = LogisticRegressionConfig {
            optimizer: OptimizationMethod::GradientDescent {
                learning_rate: 0.0,
                max_iterations: 100,
            },
            ..Default::default()
        };
        assert!(matches!(
            LogisticRegression::new(config),
            Err(LogisticRegressionError::InvalidLearningRate)
        ));
    }

    #[test]
    fn test_logistic_regression_fit_empty_dataset() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig::default()).unwrap();
        let x = Array2::<f64>::zeros((0, 2));
        let y = Array1::<f64>::zeros(0);
        assert!(matches!(
            model.fit(x.view(), y.view()),
            Err(LogisticRegressionError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_logistic_regression_fit_labels_mismatch() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y = arr1(&[1.0]);
        assert!(matches!(
            model.fit(x.view(), y.view()),
            Err(LogisticRegressionError::LabelsMismatch)
        ));
    }

    #[test]
    fn test_logistic_regression_predict_without_fit() {
        let model = LogisticRegression::new(LogisticRegressionConfig::default()).unwrap();
        let x = arr2(&[[1.0, 2.0]]);
        assert!(matches!(
            model.predict(x.view()),
            Err(LogisticRegressionError::EmptyTrainingSet)
        ));
    }

    #[test]
    fn test_logistic_regression_predict_dimension_mismatch() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig::default()).unwrap();
        let x_train = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let y_train = arr1(&[0.0, 1.0]);
        model.fit(x_train.view(), y_train.view()).unwrap();

        let x_test = arr2(&[[1.0], [2.0]]);
        assert!(matches!(
            model.predict(x_test.view()),
            Err(LogisticRegressionError::DimensionMismatch)
        ));
    }

    #[test]
    fn test_logistic_regression_binary_classification() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig {
            fit_intercept: true,
            l2_reg: 0.0,
            optimizer: OptimizationMethod::GradientDescent {
                learning_rate: 0.1,
                max_iterations: 1000,
            },
            tol: 1e-6,
        })
        .unwrap();

        // Simple binary classification dataset
        let x = arr2(&[
            [-2.0, -2.0],
            [-1.0, -1.0],
            [1.0, 1.0],
            [2.0, 2.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        model.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = model.predict(x.view()).unwrap();
        assert_eq!(predictions, y);

        // Test probabilities
        let probas = model.predict_proba(x.view()).unwrap();
        assert_eq!(probas.shape(), &[4, 2]);  // 4 samples, 2 classes
        assert!(probas.iter().all(|&p| p >= 0.0 && p <= 1.0));

        // Test loss
        let loss = model.loss(x.view(), y.view()).unwrap();
        assert!(loss >= 0.0);
    }

    #[test]
    fn test_logistic_regression_multiclass() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig {
            fit_intercept: true,
            l2_reg: 0.1,
            optimizer: OptimizationMethod::Newton {
                max_iterations: 100,
            },
            tol: 1e-6,
        })
        .unwrap();

        // Simple multiclass dataset
        let x = arr2(&[
            [0.0, 0.0],
            [0.0, 2.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [1.0, 1.0],
        ]);
        let y = arr1(&[0.0, 1.0, 2.0, 1.0, 2.0]);

        model.fit(x.view(), y.view()).unwrap();

        // Test predictions
        let predictions = model.predict(x.view()).unwrap();
        assert_eq!(predictions.len(), x.nrows());
        assert!(predictions.iter().all(|&p| p == 0.0 || p == 1.0 || p == 2.0));

        // Test probabilities
        let probas = model.predict_proba(x.view()).unwrap();
        assert_eq!(probas.shape(), &[5, 3]);  // 5 samples, 3 classes
        assert!(probas.iter().all(|&p| p >= 0.0 && p <= 1.0));

        // Each row should sum to approximately 1
        for row in probas.rows() {
            assert_relative_eq!(row.sum(), 1.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_logistic_regression_l2_regularization() {
        let mut model_with_reg = LogisticRegression::new(LogisticRegressionConfig {
            l2_reg: 1.0,
            ..Default::default()
        })
        .unwrap();

        let mut model_without_reg = LogisticRegression::new(LogisticRegressionConfig {
            l2_reg: 0.0,
            ..Default::default()
        })
        .unwrap();

        // Dataset with some noise
        let x = arr2(&[
            [-2.0, -2.0],
            [-1.9, -2.1],
            [1.9, 2.1],
            [2.0, 2.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        model_with_reg.fit(x.view(), y.view()).unwrap();
        model_without_reg.fit(x.view(), y.view()).unwrap();

        // Regularized model should have smaller coefficients
        let reg_coef_norm: f64 = model_with_reg
            .coefficients()
            .unwrap()
            .iter()
            .map(|x| x.powi(2))
            .sum();
        let unreg_coef_norm: f64 = model_without_reg
            .coefficients()
            .unwrap()
            .iter()
            .map(|x| x.powi(2))
            .sum();

        assert!(reg_coef_norm < unreg_coef_norm);
    }

    #[test]
    fn test_logistic_regression_optimization_methods() {
        // Test both GD and Newton's method on the same dataset
        let x = arr2(&[
            [-1.0, -1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [1.0, 1.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        // Gradient Descent
        let mut model_gd = LogisticRegression::new(LogisticRegressionConfig {
            optimizer: OptimizationMethod::GradientDescent {
                learning_rate: 0.1,
                max_iterations: 1000,
            },
            ..Default::default()
        })
        .unwrap();

        // Newton's Method
        let mut model_newton = LogisticRegression::new(LogisticRegressionConfig {
            optimizer: OptimizationMethod::Newton {
                max_iterations: 100,
            },
            ..Default::default()
        })
        .unwrap();

        model_gd.fit(x.view(), y.view()).unwrap();
        model_newton.fit(x.view(), y.view()).unwrap();

        // Both methods should achieve similar predictions
        let pred_gd = model_gd.predict(x.view()).unwrap();
        let pred_newton = model_newton.predict(x.view()).unwrap();

        assert_eq!(pred_gd, y);
        assert_eq!(pred_newton, y);
    }

    #[test]
    fn test_logistic_regression_convergence() {
        let mut model = LogisticRegression::new(LogisticRegressionConfig {
            optimizer: OptimizationMethod::GradientDescent {
                learning_rate: 0.1,
                max_iterations: 2,  // Very few iterations
            },
            tol: 1e-10,  // Very strict tolerance
            ..Default::default()
        })
        .unwrap();

        let x = arr2(&[
            [-2.0, -2.0],
            [-1.0, -1.0],
            [1.0, 1.0],
            [2.0, 2.0],
        ]);
        let y = arr1(&[0.0, 0.0, 1.0, 1.0]);

        // Should fail to converge with these parameters
        assert!(matches!(
            model.fit(x.view(), y.view()),
            Err(LogisticRegressionError::FailedToConverge)
        ));
    }
}