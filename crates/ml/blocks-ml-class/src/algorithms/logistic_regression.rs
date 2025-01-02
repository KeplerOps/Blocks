use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use ndarray_linalg::Solve;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogisticRegressionError {
    #[error("Empty training dataset")]
    EmptyTrainingSet,
    #[error("Empty test dataset")]
    EmptyTestSet,
    #[error("Feature dimensions mismatch")]
    DimensionMismatch,
    #[error("Labels length mismatch with training data")]
    LabelsMismatch,
    #[error("Invalid class labels")]
    InvalidLabels,
    #[error("Failed to converge within max iterations")]
    FailedToConverge,
    #[error("Invalid regularization parameter")]
    InvalidRegularization,
    #[error("Invalid learning rate")]
    InvalidLearningRate,
}

/// Optimization method for logistic regression
#[derive(Debug, Clone, Copy)]
pub enum OptimizationMethod {
    /// Gradient Descent with specified learning rate and max iterations
    GradientDescent {
        learning_rate: f64,
        max_iterations: usize,
    },
    /// Newton's Method with max iterations
    Newton {
        max_iterations: usize,
    },
}

/// Configuration for Logistic Regression
#[derive(Debug, Clone)]
pub struct LogisticRegressionConfig {
    /// Whether to fit an intercept term
    pub fit_intercept: bool,
    /// L2 regularization strength (0 for no regularization)
    pub l2_reg: f64,
    /// Optimization method to use
    pub optimizer: OptimizationMethod,
    /// Convergence tolerance
    pub tol: f64,
}

impl Default for LogisticRegressionConfig {
    fn default() -> Self {
        Self {
            fit_intercept: true,
            l2_reg: 0.0,
            optimizer: OptimizationMethod::GradientDescent {
                learning_rate: 0.1,
                max_iterations: 100,
            },
            tol: 1e-4,
        }
    }
}

/// Logistic Regression implementation for binary and multiclass classification
#[derive(Debug)]
pub struct LogisticRegression {
    config: LogisticRegressionConfig,
    coefficients: Option<Array2<f64>>,  // One column per class for multiclass
    intercepts: Option<Array1<f64>>,    // One intercept per class for multiclass
    classes: Option<Array1<f64>>,       // Unique class labels
    feature_means: Option<Array1<f64>>, // For centering features
}

impl LogisticRegression {
    /// Creates a new LogisticRegression instance with the given configuration
    pub fn new(config: LogisticRegressionConfig) -> Result<Self, LogisticRegressionError> {
        if config.l2_reg < 0.0 {
            return Err(LogisticRegressionError::InvalidRegularization);
        }
        match config.optimizer {
            OptimizationMethod::GradientDescent { learning_rate, .. } if learning_rate <= 0.0 => {
                return Err(LogisticRegressionError::InvalidLearningRate);
            }
            _ => {}
        }

        Ok(Self {
            config,
            coefficients: None,
            intercepts: None,
            classes: None,
            feature_means: None,
        })
    }

    /// Fits the logistic regression model to the training data
    pub fn fit(&mut self, x: ArrayView2<f64>, y: ArrayView1<f64>) -> Result<(), LogisticRegressionError> {
        if x.nrows() == 0 {
            return Err(LogisticRegressionError::EmptyTrainingSet);
        }
        if y.len() != x.nrows() {
            return Err(LogisticRegressionError::LabelsMismatch);
        }

        // Get unique classes and sort them
        let mut unique_classes: Vec<f64> = y.iter().copied().collect();
        unique_classes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_classes.dedup();
        self.classes = Some(Array1::from(unique_classes.clone()));

        let n_samples = x.nrows();
        let n_features = x.ncols();
        let n_classes = unique_classes.len();

        // Center features if fitting intercept
        let x_centered = if self.config.fit_intercept {
            let means = x.mean_axis(Axis(0)).unwrap();
            self.feature_means = Some(means.clone());
            let mut x_centered = x.to_owned();
            for (mut row, _) in x_centered.outer_iter_mut().zip(0..n_samples) {
                row -= &means;
            }
            x_centered
        } else {
            x.to_owned()
        };

        // Initialize coefficients and intercepts
        let mut coefficients = Array2::zeros((n_features, n_classes));
        let mut intercepts = Array1::zeros(n_classes);

        // Convert labels to one-hot encoding
        let mut y_one_hot = Array2::zeros((n_samples, n_classes));
        for (i, &label) in y.iter().enumerate() {
            let class_idx = unique_classes
                .iter()
                .position(|&c| c == label)
                .unwrap();
            y_one_hot[[i, class_idx]] = 1.0;
        }

        match self.config.optimizer {
            OptimizationMethod::GradientDescent { learning_rate, max_iterations } => {
                // Gradient Descent
                for _ in 0..max_iterations {
                    let mut old_coeffs = coefficients.clone();
                    let mut old_intercepts = intercepts.clone();

                    // Compute probabilities
                    let probas = self.compute_probabilities(&x_centered, &coefficients, &intercepts)?;

                    // Compute gradients
                    let error = &probas - &y_one_hot;
                    let coef_grad = x_centered.t().dot(&error) / n_samples as f64
                        + self.config.l2_reg * &coefficients;
                    let int_grad = error.sum_axis(Axis(0)) / n_samples as f64;

                    // Update parameters
                    coefficients -= &(learning_rate * coef_grad);
                    if self.config.fit_intercept {
                        intercepts -= learning_rate * int_grad;
                    }

                    // Check convergence
                    let coef_change = (&coefficients - &old_coeffs)
                        .iter()
                        .map(|x| x.abs())
                        .fold(0.0, f64::max);
                    let int_change = (&intercepts - &old_intercepts)
                        .iter()
                        .map(|x| x.abs())
                        .fold(0.0, f64::max);

                    if coef_change < self.config.tol && int_change < self.config.tol {
                        break;
                    }

                    if _ == max_iterations - 1 {
                        return Err(LogisticRegressionError::FailedToConverge);
                    }
                }
            }
            OptimizationMethod::Newton { max_iterations } => {
                // Newton's Method
                for _ in 0..max_iterations {
                    let mut old_coeffs = coefficients.clone();
                    let mut old_intercepts = intercepts.clone();

                    // Compute probabilities
                    let probas = self.compute_probabilities(&x_centered, &coefficients, &intercepts)?;

                    // Compute gradients
                    let error = &probas - &y_one_hot;
                    let coef_grad = x_centered.t().dot(&error) / n_samples as f64
                        + self.config.l2_reg * &coefficients;
                    let int_grad = error.sum_axis(Axis(0)) / n_samples as f64;

                    // Compute Hessian
                    let mut hessian = Array2::zeros((n_features * n_classes, n_features * n_classes));
                    for i in 0..n_samples {
                        let xi = x_centered.row(i);
                        let pi = probas.row(i);
                        let mut s = Array2::zeros((n_classes, n_classes));
                        for j in 0..n_classes {
                            for k in 0..n_classes {
                                s[[j, k]] = if j == k {
                                    pi[j] * (1.0 - pi[j])
                                } else {
                                    -pi[j] * pi[k]
                                };
                            }
                        }
                        let h = xi.t().dot(&xi) * s;
                        for j in 0..n_classes {
                            for k in 0..n_classes {
                                let jb = j * n_features;
                                let kb = k * n_features;
                                for f1 in 0..n_features {
                                    for f2 in 0..n_features {
                                        hessian[[jb + f1, kb + f2]] += h[[j, k]] * xi[f1] * xi[f2];
                                    }
                                }
                            }
                        }
                    }
                    hessian /= n_samples as f64;

                    // Add regularization to diagonal
                    for i in 0..(n_features * n_classes) {
                        hessian[[i, i]] += self.config.l2_reg;
                    }

                    // Solve Newton system
                    let grad_vec = Array1::from_iter(coef_grad.iter().copied());
                    match hessian.solve(&grad_vec) {
                        Ok(update) => {
                            // Reshape update back to coefficient matrix
                            for j in 0..n_classes {
                                for f in 0..n_features {
                                    coefficients[[f, j]] -= update[j * n_features + f];
                                }
                            }
                            if self.config.fit_intercept {
                                intercepts -= int_grad;
                            }
                        }
                        Err(_) => return Err(LogisticRegressionError::FailedToConverge),
                    }

                    // Check convergence
                    let coef_change = (&coefficients - &old_coeffs)
                        .iter()
                        .map(|x| x.abs())
                        .fold(0.0, f64::max);
                    let int_change = (&intercepts - &old_intercepts)
                        .iter()
                        .map(|x| x.abs())
                        .fold(0.0, f64::max);

                    if coef_change < self.config.tol && int_change < self.config.tol {
                        break;
                    }

                    if _ == max_iterations - 1 {
                        return Err(LogisticRegressionError::FailedToConverge);
                    }
                }
            }
        }

        self.coefficients = Some(coefficients);
        if self.config.fit_intercept {
            self.intercepts = Some(intercepts);
        }

        Ok(())
    }

    /// Predicts class labels for new data points
    pub fn predict(&self, x: ArrayView2<f64>) -> Result<Array1<f64>, LogisticRegressionError> {
        let probas = self.predict_proba(x)?;
        let classes = self.classes.as_ref().unwrap();
        
        let mut predictions = Array1::zeros(probas.nrows());
        for (i, row) in probas.outer_iter().enumerate() {
            let max_idx = row
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;
            predictions[i] = classes[max_idx];
        }
        
        Ok(predictions)
    }

    /// Predicts class probabilities for new data points
    pub fn predict_proba(&self, x: ArrayView2<f64>) -> Result<Array2<f64>, LogisticRegressionError> {
        let coefficients = self.coefficients.as_ref()
            .ok_or(LogisticRegressionError::EmptyTrainingSet)?;
        let intercepts = self.intercepts.as_ref();

        if x.ncols() != coefficients.nrows() {
            return Err(LogisticRegressionError::DimensionMismatch);
        }

        // Center features if model was fitted with intercept
        let x_centered = if let Some(means) = &self.feature_means {
            let mut x_centered = x.to_owned();
            for (mut row, _) in x_centered.outer_iter_mut().enumerate() {
                row -= means;
            }
            x_centered
        } else {
            x.to_owned()
        };

        self.compute_probabilities(&x_centered, coefficients, intercepts.unwrap_or(&Array1::zeros(0)))
    }

    /// Returns the cross-entropy loss on the given data
    pub fn loss(&self, x: ArrayView2<f64>, y: ArrayView1<f64>) -> Result<f64, LogisticRegressionError> {
        if x.nrows() == 0 {
            return Err(LogisticRegressionError::EmptyTestSet);
        }
        if y.len() != x.nrows() {
            return Err(LogisticRegressionError::LabelsMismatch);
        }

        let probas = self.predict_proba(x)?;
        let n_samples = x.nrows();
        let classes = self.classes.as_ref().unwrap();

        let mut loss = 0.0;
        for (i, &true_label) in y.iter().enumerate() {
            let class_idx = classes
                .iter()
                .position(|&c| c == true_label)
                .ok_or(LogisticRegressionError::InvalidLabels)?;
            loss -= (probas[[i, class_idx]] + 1e-15).ln();
        }
        loss /= n_samples as f64;

        // Add L2 regularization term
        if self.config.l2_reg > 0.0 {
            let coefficients = self.coefficients.as_ref().unwrap();
            let l2_term = 0.5 * self.config.l2_reg * coefficients.iter().map(|x| x.powi(2)).sum::<f64>();
            loss += l2_term;
        }

        Ok(loss)
    }

    /// Helper function to compute probabilities using softmax
    fn compute_probabilities(
        &self,
        x: &Array2<f64>,
        coefficients: &Array2<f64>,
        intercepts: &Array1<f64>,
    ) -> Result<Array2<f64>, LogisticRegressionError> {
        let n_samples = x.nrows();
        let n_classes = coefficients.ncols();
        
        // Compute linear scores
        let mut scores = x.dot(coefficients);
        if self.config.fit_intercept {
            for i in 0..n_samples {
                scores.row_mut(i).add_assign(intercepts);
            }
        }

        // Apply softmax
        let mut probas = Array2::zeros((n_samples, n_classes));
        for (mut prob_row, score_row) in probas.outer_iter_mut().zip(scores.outer_iter()) {
            let max_score = score_row.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let exp_scores: Vec<f64> = score_row.iter().map(|&s| (s - max_score).exp()).collect();
            let sum_exp = exp_scores.iter().sum::<f64>();
            for (p, &e) in prob_row.iter_mut().zip(exp_scores.iter()) {
                *p = e / sum_exp;
            }
        }

        Ok(probas)
    }

    /// Returns the model coefficients if fitted
    pub fn coefficients(&self) -> Option<&Array2<f64>> {
        self.coefficients.as_ref()
    }

    /// Returns the intercept terms if fitted
    pub fn intercepts(&self) -> Option<&Array1<f64>> {
        self.intercepts.as_ref()
    }

    /// Returns the unique class labels if fitted
    pub fn classes(&self) -> Option<&Array1<f64>> {
        self.classes.as_ref()
    }
}