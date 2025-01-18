use ndarray::{Array1, Array2};

use crate::error::Result;

/// Core trait for supervised learning algorithms
pub trait Supervised {
    /// Fit the model to training data
    fn fit(&mut self, x: &Array2<f64>, y: &Array1<f64>) -> Result<()>;
    
    /// Predict output for new data
    fn predict(&self, x: &Array2<f64>) -> Result<Array1<f64>>;
}

/// Core trait for unsupervised learning algorithms
pub trait Unsupervised {
    /// Fit the model to training data
    fn fit(&mut self, x: &Array2<f64>) -> Result<()>;
    
    /// Transform or cluster new data
    fn transform(&self, x: &Array2<f64>) -> Result<Array2<f64>>;
}

/// Optional trait for algorithms that support online learning
pub trait OnlineLearning {
    /// Update the model with new training data
    fn partial_fit(&mut self, x: &Array2<f64>, y: &Array1<f64>) -> Result<()>;
}

/// Optional trait for algorithms that provide feature importance
pub trait FeatureImportance {
    /// Get feature importance scores
    fn feature_importance(&self) -> Result<Array1<f64>>;
}