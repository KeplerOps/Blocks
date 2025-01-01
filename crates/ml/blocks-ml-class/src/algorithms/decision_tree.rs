use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecisionTreeError {
    #[error("Empty training dataset")]
    EmptyTrainingSet,
    #[error("Empty test dataset")]
    EmptyTestSet,
    #[error("Feature dimensions mismatch")]
    DimensionMismatch,
    #[error("Labels length mismatch with training data")]
    LabelsMismatch,
    #[error("Invalid feature index")]
    InvalidFeatureIndex,
    #[error("Invalid split threshold")]
    InvalidSplitThreshold,
    #[error("Invalid pruning parameters")]
    InvalidPruningParams,
}

/// Split criterion for decision tree
#[derive(Debug, Clone, Copy)]
pub enum SplitCriterion {
    /// Information Gain (ID3)
    InformationGain,
    /// Gain Ratio (C4.5)
    GainRatio,
}

/// Node type in the decision tree
#[derive(Debug)]
enum Node {
    /// Leaf node with predicted class label and sample count
    Leaf {
        prediction: f64,
        n_samples: usize,
    },
    /// Internal node with split information
    Internal {
        feature_idx: usize,
        threshold: f64,
        left: Box<Node>,
        right: Box<Node>,
        n_samples: usize,
    },
}

/// Configuration for Decision Tree
#[derive(Debug, Clone)]
pub struct DecisionTreeConfig {
    /// Maximum depth of the tree
    pub max_depth: Option<usize>,
    /// Minimum samples required to split a node
    pub min_samples_split: usize,
    /// Minimum samples required in a leaf node
    pub min_samples_leaf: usize,
    /// Split criterion to use
    pub criterion: SplitCriterion,
    /// Maximum features to consider for best split
    pub max_features: Option<usize>,
    /// Minimum impurity decrease required for split
    pub min_impurity_decrease: f64,
}

impl Default for DecisionTreeConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            min_samples_split: 2,
            min_samples_leaf: 1,
            criterion: SplitCriterion::GainRatio,
            max_features: None,
            min_impurity_decrease: 0.0,
        }
    }
}

/// Decision Tree implementation supporting both ID3 and C4.5 algorithms
#[derive(Debug)]
pub struct DecisionTree {
    config: DecisionTreeConfig,
    root: Option<Node>,
    n_features: usize,
    feature_importances: Option<Array1<f64>>,
}

impl DecisionTree {
    /// Creates a new DecisionTree instance with the given configuration
    pub fn new(config: DecisionTreeConfig) -> Result<Self, DecisionTreeError> {
        if config.min_samples_split < 2 {
            return Err(DecisionTreeError::InvalidPruningParams);
        }
        if config.min_samples_leaf < 1 {
            return Err(DecisionTreeError::InvalidPruningParams);
        }
        if config.min_impurity_decrease < 0.0 {
            return Err(DecisionTreeError::InvalidPruningParams);
        }

        Ok(Self {
            config,
            root: None,
            n_features: 0,
            feature_importances: None,
        })
    }

    /// Fits the decision tree to the training data
    pub fn fit(&mut self, x: ArrayView2<f64>, y: ArrayView1<f64>) -> Result<(), DecisionTreeError> {
        if x.nrows() == 0 {
            return Err(DecisionTreeError::EmptyTrainingSet);
        }
        if y.len() != x.nrows() {
            return Err(DecisionTreeError::LabelsMismatch);
        }

        self.n_features = x.ncols();
        let mut feature_importances = Array1::zeros(self.n_features);
        let n_samples = x.nrows();

        // Build tree recursively
        self.root = Some(self.build_tree(
            x,
            y,
            0,
            n_samples,
            &mut feature_importances,
            0,
        )?);

        // Normalize feature importances
        let total_importance = feature_importances.sum();
        if total_importance > 0.0 {
            feature_importances.mapv_inplace(|x| x / total_importance);
        }
        self.feature_importances = Some(feature_importances);

        Ok(())
    }

    /// Predicts class labels for new data points
    pub fn predict(&self, x: ArrayView2<f64>) -> Result<Array1<f64>, DecisionTreeError> {
        if x.ncols() != self.n_features {
            return Err(DecisionTreeError::DimensionMismatch);
        }

        let root = self.root.as_ref().ok_or(DecisionTreeError::EmptyTrainingSet)?;
        let mut predictions = Array1::zeros(x.nrows());

        for (i, sample) in x.outer_iter().enumerate() {
            predictions[i] = self.predict_single(root, sample);
        }

        Ok(predictions)
    }

    /// Returns feature importances if the tree is fitted
    pub fn feature_importances(&self) -> Option<&Array1<f64>> {
        self.feature_importances.as_ref()
    }

    /// Returns the depth of the tree
    pub fn depth(&self) -> usize {
        match &self.root {
            Some(root) => self.node_depth(root),
            None => 0,
        }
    }

    /// Returns the number of nodes in the tree
    pub fn node_count(&self) -> usize {
        match &self.root {
            Some(root) => self.count_nodes(root),
            None => 0,
        }
    }

    /// Returns the number of leaves in the tree
    pub fn leaf_count(&self) -> usize {
        match &self.root {
            Some(root) => self.count_leaves(root),
            None => 0,
        }
    }

    // Helper methods
    fn build_tree(
        &self,
        x: ArrayView2<f64>,
        y: ArrayView1<f64>,
        start: usize,
        end: usize,
        feature_importances: &mut Array1<f64>,
        depth: usize,
    ) -> Result<Node, DecisionTreeError> {
        let n_samples = end - start;

        // Check stopping criteria
        if n_samples < self.config.min_samples_split
            || (self.config.max_depth.is_some() && depth >= self.config.max_depth.unwrap())
        {
            return Ok(self.create_leaf(y.slice(s![start..end])));
        }

        // Find best split
        let (feature_idx, threshold, impurity_decrease) = self.find_best_split(x.slice(s![start..end, ..]), y.slice(s![start..end]))?;

        // Check if split is good enough
        if impurity_decrease < self.config.min_impurity_decrease {
            return Ok(self.create_leaf(y.slice(s![start..end])));
        }

        // Update feature importance
        feature_importances[feature_idx] += impurity_decrease * n_samples as f64;

        // Split data
        let mut left_indices = Vec::new();
        let mut right_indices = Vec::new();
        for i in start..end {
            if x[[i, feature_idx]] <= threshold {
                left_indices.push(i);
            } else {
                right_indices.push(i);
            }
        }

        // Check min_samples_leaf
        if left_indices.len() < self.config.min_samples_leaf
            || right_indices.len() < self.config.min_samples_leaf
        {
            return Ok(self.create_leaf(y.slice(s![start..end])));
        }

        // Build subtrees
        let left = Box::new(self.build_tree(
            x,
            y,
            start,
            start + left_indices.len(),
            feature_importances,
            depth + 1,
        )?);
        let right = Box::new(self.build_tree(
            x,
            y,
            start + left_indices.len(),
            end,
            feature_importances,
            depth + 1,
        )?);

        Ok(Node::Internal {
            feature_idx,
            threshold,
            left,
            right,
            n_samples,
        })
    }

    fn find_best_split(
        &self,
        x: ArrayView2<f64>,
        y: ArrayView1<f64>,
    ) -> Result<(usize, f64, f64), DecisionTreeError> {
        let n_features = x.ncols();
        let mut best_feature = 0;
        let mut best_threshold = 0.0;
        let mut best_impurity_decrease = f64::NEG_INFINITY;

        // Randomly select features to consider
        let n_features_to_consider = self.config.max_features.unwrap_or(n_features);
        let mut feature_indices: Vec<usize> = (0..n_features).collect();
        feature_indices.shuffle(&mut rand::thread_rng());
        let feature_indices = &feature_indices[..n_features_to_consider];

        for &feature_idx in feature_indices {
            // Get unique values for the feature
            let mut values: Vec<f64> = x.column(feature_idx).to_vec();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            values.dedup();

            // Try each value as a threshold
            for i in 0..values.len() - 1 {
                let threshold = (values[i] + values[i + 1]) / 2.0;
                let impurity_decrease = self.compute_impurity_decrease(x.column(feature_idx), y, threshold)?;

                if impurity_decrease > best_impurity_decrease {
                    best_feature = feature_idx;
                    best_threshold = threshold;
                    best_impurity_decrease = impurity_decrease;
                }
            }
        }

        Ok((best_feature, best_threshold, best_impurity_decrease))
    }

    fn compute_impurity_decrease(
        &self,
        feature: ArrayView1<f64>,
        y: ArrayView1<f64>,
        threshold: f64,
    ) -> Result<f64, DecisionTreeError> {
        let n_samples = y.len() as f64;
        let mut left_counts = HashMap::new();
        let mut right_counts = HashMap::new();
        let mut n_left = 0.0;
        let mut n_right = 0.0;

        // Count samples in each split
        for (val, &label) in feature.iter().zip(y.iter()) {
            if *val <= threshold {
                *left_counts.entry(label).or_insert(0.0) += 1.0;
                n_left += 1.0;
            } else {
                *right_counts.entry(label).or_insert(0.0) += 1.0;
                n_right += 1.0;
            }
        }

        // Compute impurity decrease based on criterion
        match self.config.criterion {
            SplitCriterion::InformationGain => {
                let parent_entropy = self.compute_entropy(&y);
                let left_entropy = self.compute_entropy_from_counts(&left_counts, n_left);
                let right_entropy = self.compute_entropy_from_counts(&right_counts, n_right);
                let weighted_child_entropy = (n_left * left_entropy + n_right * right_entropy) / n_samples;
                parent_entropy - weighted_child_entropy
            }
            SplitCriterion::GainRatio => {
                let info_gain = {
                    let parent_entropy = self.compute_entropy(&y);
                    let left_entropy = self.compute_entropy_from_counts(&left_counts, n_left);
                    let right_entropy = self.compute_entropy_from_counts(&right_counts, n_right);
                    let weighted_child_entropy = (n_left * left_entropy + n_right * right_entropy) / n_samples;
                    parent_entropy - weighted_child_entropy
                };
                let split_info = -(n_left / n_samples * (n_left / n_samples).ln()
                    + n_right / n_samples * (n_right / n_samples).ln());
                if split_info == 0.0 {
                    0.0
                } else {
                    info_gain / split_info
                }
            }
        }
    }

    fn compute_entropy(&self, y: &ArrayView1<f64>) -> f64 {
        let n_samples = y.len() as f64;
        let mut counts = HashMap::new();
        for &label in y.iter() {
            *counts.entry(label).or_insert(0.0) += 1.0;
        }
        self.compute_entropy_from_counts(&counts, n_samples)
    }

    fn compute_entropy_from_counts(&self, counts: &HashMap<f64, f64>, n_samples: f64) -> f64 {
        -counts
            .values()
            .map(|&count| {
                let p = count / n_samples;
                if p > 0.0 {
                    p * p.ln()
                } else {
                    0.0
                }
            })
            .sum::<f64>()
    }

    fn create_leaf(&self, y: ArrayView1<f64>) -> Node {
        let n_samples = y.len();
        let mut counts = HashMap::new();
        for &label in y.iter() {
            *counts.entry(label).or_insert(0) += 1;
        }
        let prediction = counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .unwrap()
            .0;
        Node::Leaf {
            prediction,
            n_samples,
        }
    }

    fn predict_single(&self, node: &Node, sample: ArrayView1<f64>) -> f64 {
        match node {
            Node::Leaf { prediction, .. } => *prediction,
            Node::Internal {
                feature_idx,
                threshold,
                left,
                right,
                ..
            } => {
                if sample[*feature_idx] <= *threshold {
                    self.predict_single(left, sample)
                } else {
                    self.predict_single(right, sample)
                }
            }
        }
    }

    fn node_depth(&self, node: &Node) -> usize {
        match node {
            Node::Leaf { .. } => 0,
            Node::Internal { left, right, .. } => {
                1 + std::cmp::max(self.node_depth(left), self.node_depth(right))
            }
        }
    }

    fn count_nodes(&self, node: &Node) -> usize {
        match node {
            Node::Leaf { .. } => 1,
            Node::Internal { left, right, .. } => {
                1 + self.count_nodes(left) + self.count_nodes(right)
            }
        }
    }

    fn count_leaves(&self, node: &Node) -> usize {
        match node {
            Node::Leaf { .. } => 1,
            Node::Internal { left, right, .. } => {
                self.count_leaves(left) + self.count_leaves(right)
            }
        }
    }
}