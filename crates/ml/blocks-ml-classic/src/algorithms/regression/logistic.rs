use ndarray::{s, Array1, Array2, Axis};
use ndarray_linalg::Solve;

use crate::error::{Error, Result};
use crate::traits::Supervised;

/// Logistic Regression for binary classification
#[derive(Debug)]
pub struct LogisticRegression {
    coefficients: Option<Array1<f64>>,
    intercept: Option<f64>,
    fit_intercept: bool,
    alpha: f64,  // L2 regularization parameter
    max_iter: usize,
    tol: f64,
}

impl Default for LogisticRegression {
    fn default() -> Self {
        Self {
            coefficients: None,
            intercept: None,
            fit_intercept: true,
            alpha: 1e-4,
            max_iter: 100,
            tol: 1e-4,
        }
    }
}

impl LogisticRegression {
    /// Create a new LogisticRegression instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to fit an intercept term
    pub fn fit_intercept(mut self, fit_intercept: bool) -> Self {
        self.fit_intercept = fit_intercept;
        self
    }

    /// Set the L2 regularization parameter (alpha)
    pub fn alpha(mut self, alpha: f64) -> Self {
        if alpha < 0.0 {
            panic!("alpha must be non-negative");
        }
        self.alpha = alpha;
        self
    }

    /// Set the maximum number of iterations
    pub fn max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Set the convergence tolerance
    pub fn tol(mut self, tol: f64) -> Self {
        if tol <= 0.0 {
            panic!("tolerance must be positive");
        }
        self.tol = tol;
        self
    }

    /// Get the fitted coefficients (weights)
    pub fn coefficients(&self) -> Option<&Array1<f64>> {
        self.coefficients.as_ref()
    }

    /// Get the fitted intercept term
    pub fn intercept(&self) -> Option<f64> {
        self.intercept
    }

    /// Calculate the sigmoid function: σ(x) = 1 / (1 + e^(-x))
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Calculate the predicted probabilities: P(y=1|X)
    fn predict_proba(&self, x: &Array2<f64>) -> Result<Array1<f64>> {
        let coefficients = self.coefficients.as_ref().ok_or_else(|| {
            Error::InvalidState("Model must be fitted before prediction".to_string())
        })?;

        let intercept = self.intercept.ok_or_else(|| {
            Error::InvalidState("Model must be fitted before prediction".to_string())
        })?;

        if x.ncols() != coefficients.len() {
            return Err(Error::InvalidDimensions {
                expected: vec![coefficients.len()],
                got: vec![x.ncols()],
            });
        }

        let z = x.dot(coefficients) + intercept;
        Ok(z.mapv(Self::sigmoid))
    }

    /// Calculate accuracy score
    pub fn score(&self, x: &Array2<f64>, y: &Array1<f64>) -> Result<f64> {
        let y_pred = self.predict(x)?;
        let correct = y_pred.iter().zip(y.iter())
            .filter(|(&p, &t)| (p - t).abs() < 1e-10)
            .count();
        Ok(correct as f64 / y.len() as f64)
    }
}

impl Supervised for LogisticRegression {
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

        // Check if y contains only 0s and 1s
        if !y.iter().all(|&yi| (yi - 0.0).abs() < 1e-10 || (yi - 1.0).abs() < 1e-10) {
            return Err(Error::InvalidParameter(
                "Target values must be 0 or 1 for binary classification".to_string(),
            ));
        }

        // Add bias term if fitting intercept
        let x_with_bias = if self.fit_intercept {
            let mut x_bias = Array2::ones((x.nrows(), x.ncols() + 1));
            x_bias.slice_mut(s![.., 1..]).assign(x);
            x_bias
        } else {
            x.to_owned()
        };

        // Scale features to improve conditioning
        let mut x_scaled = x_with_bias.clone();
        let mut scale = Array1::ones(x_scaled.ncols());
        
        // Skip scaling the bias column if present
        let start_col = if self.fit_intercept { 1 } else { 0 };
        for (i, mut col) in x_scaled.columns_mut().into_iter().enumerate().skip(start_col) {
            let std = col.std(0.0);
            if std > 0.0 {
                scale[i] = 1.0 / std;
                col.mapv_inplace(|x| x * scale[i]);
            }
        }

        // Initialize coefficients
        let mut beta = Array1::zeros(x_scaled.ncols());
        let mut prev_beta = beta.clone();

        // Newton-Raphson optimization
        for _ in 0..self.max_iter {
            // Calculate predictions
            let z = x_scaled.dot(&beta);
            let h = z.mapv(Self::sigmoid);

            // Calculate gradient: X'(h - y) + αβ
            let error = &h - y;
            let mut grad = x_scaled.t().dot(&error);
            for i in 0..grad.len() {
                grad[i] += self.alpha * beta[i];
            }

            // Calculate Hessian: X'WX + αI where W = diag(h(1-h))
            let w = h.mapv(|hi| hi * (1.0 - hi));
            let mut hessian = Array2::zeros((x_scaled.ncols(), x_scaled.ncols()));
            
            // Compute X'WX
            for i in 0..x_scaled.ncols() {
                for j in 0..x_scaled.ncols() {
                    let mut sum = 0.0;
                    for k in 0..x_scaled.nrows() {
                        sum += x_scaled[[k, i]] * w[k] * x_scaled[[k, j]];
                    }
                    hessian[[i, j]] = sum;
                }
            }

            // Add regularization term
            for i in 0..hessian.nrows() {
                hessian[[i, i]] += self.alpha;
            }

            // Update coefficients: β = β - H⁻¹g
            let delta = hessian.solve_into(grad)?;
            beta = beta - delta;

            // Check convergence
            let change = (&beta - &prev_beta).mapv(|x| x.abs()).sum() / beta.len() as f64;
            if change < self.tol {
                break;
            }
            prev_beta = beta.clone();
        }

        // Unscale coefficients
        for i in 0..beta.len() {
            beta[i] *= scale[i];
        }

        if self.fit_intercept {
            self.intercept = Some(beta[0]);
            self.coefficients = Some(beta.slice(s![1..]).to_owned());
        } else {
            self.intercept = Some(0.0);
            self.coefficients = Some(beta);
        }

        Ok(())
    }

    fn predict(&self, x: &Array2<f64>) -> Result<Array1<f64>> {
        let proba = self.predict_proba(x)?;
        Ok(proba.mapv(|p| if p >= 0.5 { 1.0 } else { 0.0 }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_logistic_regression_perfect_separation() {
        // Create linearly separable data
        let x = Array2::from_shape_vec((8, 2), vec![
            0.0, 0.0,  // Class 0 points
            0.0, 1.0,
            1.0, 0.0,
            1.0, 1.0,
            2.0, 2.0,  // Class 1 points
            2.0, 3.0,
            3.0, 2.0,
            3.0, 3.0,
        ]).unwrap();
        let y = Array1::from_vec(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);

        let mut model = LogisticRegression::new().alpha(1e-8);
        model.fit(&x, &y).unwrap();

        // Test predictions
        let y_pred = model.predict(&x).unwrap();
        assert_eq!(y_pred.to_vec(), y.to_vec());

        // Test accuracy
        let accuracy = model.score(&x, &y).unwrap();
        assert_relative_eq!(accuracy, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_logistic_regression_probability_range() {
        // Create simple dataset
        let x = Array2::from_shape_vec((4, 1), vec![0.0, 1.0, 2.0, 3.0]).unwrap();
        let y = Array1::from_vec(vec![0.0, 0.0, 1.0, 1.0]);

        let mut model = LogisticRegression::new();
        model.fit(&x, &y).unwrap();

        // Test that predicted probabilities are in [0, 1]
        let proba = model.predict_proba(&x).unwrap();
        for p in proba.iter() {
            assert!(*p >= 0.0 && *p <= 1.0);
        }
    }

    #[test]
    fn test_logistic_regression_no_intercept() {
        // Create simple dataset where decision boundary passes through origin
        let x = Array2::from_shape_vec((6, 2), vec![
            -1.0, -1.0,  // Class 0 points (x + y < 0)
            -2.0, 1.0,
            1.0, -2.0,
            1.0, 1.0,    // Class 1 points (x + y > 0)
            2.0, -1.0,
            -1.0, 2.0,
        ]).unwrap();
        let y = Array1::from_vec(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        let mut model = LogisticRegression::new()
            .fit_intercept(false)
            .alpha(1e-8);
        model.fit(&x, &y).unwrap();

        assert_eq!(model.intercept(), Some(0.0));

        // Test predictions
        let y_pred = model.predict(&x).unwrap();
        assert_eq!(y_pred.to_vec(), y.to_vec());
    }

    #[test]
    fn test_logistic_regression_invalid_target() {
        let x = Array2::zeros((3, 2));
        let y = Array1::from_vec(vec![0.0, 0.5, 1.0]); // Invalid: contains 0.5

        let mut model = LogisticRegression::new();
        assert!(model.fit(&x, &y).is_err());
    }

    #[test]
    fn test_logistic_regression_empty_input() {
        let x = Array2::zeros((0, 2));
        let y = Array1::zeros(0);
        let mut model = LogisticRegression::new();
        assert!(model.fit(&x, &y).is_err());
    }

    #[test]
    fn test_logistic_regression_dimension_mismatch() {
        let x = Array2::zeros((10, 2));
        let y = Array1::zeros(5);
        let mut model = LogisticRegression::new();
        assert!(model.fit(&x, &y).is_err());
    }

    #[test]
    fn test_logistic_regression_predict_without_fit() {
        let x = Array2::zeros((3, 2));
        let model = LogisticRegression::new();
        assert!(model.predict(&x).is_err());
    }

    #[test]
    fn test_logistic_regression_predict_wrong_dimensions() {
        let x_train = Array2::zeros((5, 2));
        let y_train = Array1::zeros(5);
        let mut model = LogisticRegression::new();
        model.fit(&x_train, &y_train).unwrap();

        let x_test = Array2::zeros((3, 3)); // Wrong number of features
        assert!(model.predict(&x_test).is_err());
    }
}