use ndarray::{s, Array1, Array2, Axis, ShapeError};
use ndarray_linalg::{error::LinalgError, Solve};

use crate::error::{Error, Result};

impl From<LinalgError> for Error {
    fn from(err: LinalgError) -> Self {
        Error::NumericalError(err.to_string())
    }
}

impl From<ShapeError> for Error {
    fn from(_: ShapeError) -> Self {
        Error::InvalidDimensions {
            expected: vec![],
            got: vec![],
        }
    }
}
use crate::traits::Supervised;

/// Linear Regression using Ordinary Least Squares (OLS)
#[derive(Debug)]
pub struct LinearRegression {
    coefficients: Option<Array1<f64>>,
    intercept: Option<f64>,
    fit_intercept: bool,
    alpha: f64,  // L2 regularization parameter
}

impl Default for LinearRegression {
    fn default() -> Self {
        Self {
            coefficients: None,
            intercept: None,
            fit_intercept: true,
            alpha: 1e-10,  // Small regularization by default
        }
    }
}

impl LinearRegression {
    /// Create a new LinearRegression instance
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

    /// Get the fitted coefficients (weights)
    pub fn coefficients(&self) -> Option<&Array1<f64>> {
        self.coefficients.as_ref()
    }

    /// Get the fitted intercept term
    pub fn intercept(&self) -> Option<f64> {
        self.intercept
    }

    /// Calculate R² score (coefficient of determination)
    pub fn score(&self, x: &Array2<f64>, y: &Array1<f64>) -> Result<f64> {
        let y_pred = self.predict(x)?;
        let y_mean = y.mean().unwrap();
        
        let ss_tot = y.iter()
            .map(|&yi| (yi - y_mean).powi(2))
            .sum::<f64>();
        
        let ss_res = y.iter()
            .zip(y_pred.iter())
            .map(|(&yi, &y_pred_i)| (yi - y_pred_i).powi(2))
            .sum::<f64>();
        
        Ok(1.0 - (ss_res / ss_tot))
    }
}

impl Supervised for LinearRegression {
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

        let (n_samples, n_features) = x.dim();
        
        // Add bias term if fitting intercept
        let x_with_bias = if self.fit_intercept {
            let mut x_bias = Array2::ones((n_samples, n_features + 1));
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

        // Solve OLS using normal equations with regularization
        // (X'X + αI)β = X'y
        let xt = x_scaled.t();
        let mut xtx = xt.dot(&x_scaled);
        
        // Add regularization term
        for i in 0..xtx.nrows() {
            xtx[[i, i]] += self.alpha;
        }
        
        // Convert y to column vector and compute X'y
        let y_col = y.clone().into_shape((y.len(), 1))?;
        let xty = xt.dot(&y_col).into_shape(xtx.nrows())?;
        
        // Solve for β and unscale
        let mut beta = xtx.solve_into(xty)?;
        
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

        // y = Xβ + b
        let predictions = x.dot(coefficients) + intercept;
        Ok(predictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_linear_regression_perfect_fit() {
        // y = 2x + 1
        let x = Array2::from_shape_vec((5, 1), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let y = Array1::from_vec(vec![3.0, 5.0, 7.0, 9.0, 11.0]);

        let mut model = LinearRegression::new();
        model.fit(&x, &y).unwrap();

        let coef = model.coefficients().unwrap();
        let intercept = model.intercept().unwrap();

        assert_relative_eq!(coef[0], 2.0, epsilon = 1e-10);
        assert_relative_eq!(intercept, 1.0, epsilon = 1e-10);

        // Test predictions
        let x_test = Array2::from_shape_vec((2, 1), vec![0.0, 6.0]).unwrap();
        let predictions = model.predict(&x_test).unwrap();
        
        assert_relative_eq!(predictions[0], 1.0, epsilon = 1e-10); // 2*0 + 1
        assert_relative_eq!(predictions[1], 13.0, epsilon = 1e-10); // 2*6 + 1
    }

    #[test]
    fn test_linear_regression_multiple_features() {
        // y = 2x₁ + 3x₂ + 1
        let x = Array2::from_shape_vec((20, 2), vec![
            0.0, 0.0,  // Origin point
            1.0, 0.0,  // Unit points
            0.0, 1.0,
            -1.0, 0.0,
            0.0, -1.0,
            1.0, 1.0,  // Diagonal points
            -1.0, -1.0,
            1.0, -1.0,
            -1.0, 1.0,
            2.0, 0.0,  // Axis-aligned points
            0.0, 2.0,
            -2.0, 0.0,
            0.0, -2.0,
            2.0, 2.0,  // More diagonal points
            -2.0, -2.0,
            2.0, -2.0,
            -2.0, 2.0,
            3.0, 0.0,  // Additional points
            0.0, 3.0,
            1.0, 2.0,
        ]).unwrap();

        // Calculate y values: y = 2x₁ + 3x₂ + 1
        let y = Array1::from_shape_fn(x.nrows(), |i| {
            2.0 * x[[i, 0]] + 3.0 * x[[i, 1]] + 1.0
        });

        let mut model = LinearRegression::new().alpha(1e-10); // Small regularization for numerical stability
        model.fit(&x, &y).unwrap();

        let coef = model.coefficients().unwrap();
        let intercept = model.intercept().unwrap();

        assert_relative_eq!(coef[0], 2.0, epsilon = 1e-4);
        assert_relative_eq!(coef[1], 3.0, epsilon = 1e-4);
        assert_relative_eq!(intercept, 1.0, epsilon = 1e-4);

        // Test R² score (should be 1.0 for perfect fit)
        let r2 = model.score(&x, &y).unwrap();
        assert_relative_eq!(r2, 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_linear_regression_no_intercept() {
        // y = 2x (no intercept)
        let x = Array2::from_shape_vec((5, 1), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let y = Array1::from_vec(vec![2.0, 4.0, 6.0, 8.0, 10.0]);

        let mut model = LinearRegression::new().fit_intercept(false);
        model.fit(&x, &y).unwrap();

        let coef = model.coefficients().unwrap();
        let intercept = model.intercept().unwrap();

        assert_relative_eq!(coef[0], 2.0, epsilon = 1e-10);
        assert_relative_eq!(intercept, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_linear_regression_empty_input() {
        let x = Array2::zeros((0, 2));
        let y = Array1::zeros(0);
        let mut model = LinearRegression::new();
        assert!(model.fit(&x, &y).is_err());
    }

    #[test]
    fn test_linear_regression_dimension_mismatch() {
        let x = Array2::zeros((10, 2));
        let y = Array1::zeros(5);
        let mut model = LinearRegression::new();
        assert!(model.fit(&x, &y).is_err());
    }

    #[test]
    fn test_linear_regression_predict_without_fit() {
        let x = Array2::zeros((3, 2));
        let model = LinearRegression::new();
        assert!(model.predict(&x).is_err());
    }

    #[test]
    fn test_linear_regression_predict_wrong_dimensions() {
        let x_train = Array2::zeros((5, 2));
        let y_train = Array1::zeros(5);
        let mut model = LinearRegression::new();
        model.fit(&x_train, &y_train).unwrap();

        let x_test = Array2::zeros((3, 3)); // Wrong number of features
        assert!(model.predict(&x_test).is_err());
    }

    #[test]
    fn test_linear_regression_noisy_data() {
        // y = 2x + 1 + noise
        let x = Array2::from_shape_vec((5, 1), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let y = Array1::from_vec(vec![3.1, 4.9, 7.2, 8.8, 11.1]); // Added some noise

        let mut model = LinearRegression::new();
        model.fit(&x, &y).unwrap();

        let coef = model.coefficients().unwrap();
        let intercept = model.intercept().unwrap();

        // Check that coefficients are close to true values
        assert_relative_eq!(coef[0], 2.0, epsilon = 0.1);
        assert_relative_eq!(intercept, 1.0, epsilon = 0.1);

        // R² should be slightly less than 1.0 due to noise
        let r2 = model.score(&x, &y).unwrap();
        assert!(r2 > 0.99); // Very good fit but not perfect
        assert!(r2 < 1.0);
    }
}