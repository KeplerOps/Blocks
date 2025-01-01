use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use num_traits::Float;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KMeansError {
    #[error("Number of clusters must be greater than 0")]
    InvalidClusters,
    #[error("Empty dataset provided")]
    EmptyDataset,
    #[error("Maximum iterations must be greater than 0")]
    InvalidMaxIterations,
    #[error("Convergence threshold must be positive")]
    InvalidConvergenceThreshold,
    #[error("Failed to converge within maximum iterations")]
    FailedToConverge,
}

/// Configuration parameters for the k-means algorithm
#[derive(Debug, Clone)]
pub struct KMeansConfig {
    /// Number of clusters (k)
    pub n_clusters: usize,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence threshold for centroid movement
    pub convergence_threshold: f64,
    /// Random seed for initialization
    pub random_seed: Option<u64>,
}

impl Default for KMeansConfig {
    fn default() -> Self {
        Self {
            n_clusters: 8,
            max_iterations: 300,
            convergence_threshold: 1e-4,
            random_seed: None,
        }
    }
}

/// K-means clustering algorithm implementation
#[derive(Debug)]
pub struct KMeans {
    config: KMeansConfig,
    centroids: Option<Array2<f64>>,
}

impl KMeans {
    /// Creates a new KMeans instance with the given configuration
    pub fn new(config: KMeansConfig) -> Result<Self, KMeansError> {
        if config.n_clusters == 0 {
            return Err(KMeansError::InvalidClusters);
        }
        if config.max_iterations == 0 {
            return Err(KMeansError::InvalidMaxIterations);
        }
        if config.convergence_threshold <= 0.0 {
            return Err(KMeansError::InvalidConvergenceThreshold);
        }

        Ok(Self {
            config,
            centroids: None,
        })
    }

    /// Fits the k-means model to the provided data
    pub fn fit(&mut self, data: ArrayView2<f64>) -> Result<(), KMeansError> {
        if data.nrows() == 0 {
            return Err(KMeansError::EmptyDataset);
        }

        let n_samples = data.nrows();
        let n_features = data.ncols();

        // Initialize centroids using k-means++ initialization
        self.centroids = Some(self.initialize_centroids(data)?);
        let mut old_centroids = Array2::zeros((self.config.n_clusters, n_features));
        let mut labels = Array1::zeros(n_samples);

        for iteration in 0..self.config.max_iterations {
            // Assign points to nearest centroids
            self.assign_clusters(data, &mut labels)?;

            // Update centroids
            let centroids = self.centroids.as_mut().unwrap();
            old_centroids.assign(centroids);
            self.update_centroids(data, &labels, centroids)?;

            // Check convergence
            let max_centroid_shift = (0..self.config.n_clusters)
                .map(|i| {
                    euclidean_distance(
                        old_centroids.row(i),
                        centroids.row(i),
                    )
                })
                .fold(0.0, f64::max);

            if max_centroid_shift < self.config.convergence_threshold {
                return Ok(());
            }
        }

        Err(KMeansError::FailedToConverge)
    }

    /// Predicts cluster assignments for new data points
    pub fn predict(&self, data: ArrayView2<f64>) -> Result<Array1<usize>, KMeansError> {
        let centroids = self.centroids.as_ref()
            .ok_or_else(|| KMeansError::EmptyDataset)?;
        
        let mut labels = Array1::zeros(data.nrows());
        self.assign_clusters(data, &mut labels)?;
        Ok(labels)
    }

    /// Fits the model and predicts cluster assignments in one step
    pub fn fit_predict(&mut self, data: ArrayView2<f64>) -> Result<Array1<usize>, KMeansError> {
        self.fit(data)?;
        self.predict(data)
    }

    /// Returns the current centroids if the model has been fitted
    pub fn centroids(&self) -> Option<&Array2<f64>> {
        self.centroids.as_ref()
    }

    /// Computes the inertia (within-cluster sum of squares) for the current model
    pub fn inertia(&self, data: ArrayView2<f64>) -> Result<f64, KMeansError> {
        let labels = self.predict(data)?;
        let mut total_inertia = 0.0;

        for (point, &label) in data.outer_iter().zip(labels.iter()) {
            let centroid = self.centroids
                .as_ref()
                .unwrap()
                .row(label);
            total_inertia += euclidean_distance(point, centroid).powi(2);
        }

        Ok(total_inertia)
    }

    // Helper methods
    fn initialize_centroids(&self, data: ArrayView2<f64>) -> Result<Array2<f64>, KMeansError> {
        let n_samples = data.nrows();
        let n_features = data.ncols();
        let mut rng = match self.config.random_seed {
            Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
            None => rand::rngs::StdRng::from_entropy(),
        };

        let mut centroids = Array2::zeros((self.config.n_clusters, n_features));
        let mut distances = Array1::zeros(n_samples);

        // Choose first centroid randomly
        let first_idx = rand::Rng::gen_range(&mut rng, 0..n_samples);
        centroids
            .row_mut(0)
            .assign(&data.row(first_idx));

        // Choose remaining centroids using k-means++ initialization
        for k in 1..self.config.n_clusters {
            // Compute distances to nearest centroid for each point
            for (i, point) in data.outer_iter().enumerate() {
                let min_dist = (0..k)
                    .map(|j| euclidean_distance(point, centroids.row(j)))
                    .fold(f64::INFINITY, f64::min);
                distances[i] = min_dist.powi(2);
            }

            // Normalize distances to create probability distribution
            let total_dist: f64 = distances.sum();
            if total_dist == 0.0 {
                // If all points are identical, randomly choose remaining centroids
                for j in k..self.config.n_clusters {
                    let idx = rand::Rng::gen_range(&mut rng, 0..n_samples);
                    centroids.row_mut(j).assign(&data.row(idx));
                }
                break;
            }

            // Choose next centroid with probability proportional to distance squared
            let mut cumsum = 0.0;
            let threshold = rand::Rng::gen::<f64>(&mut rng) * total_dist;
            
            for (i, &dist) in distances.iter().enumerate() {
                cumsum += dist;
                if cumsum >= threshold {
                    centroids.row_mut(k).assign(&data.row(i));
                    break;
                }
            }
        }

        Ok(centroids)
    }

    fn assign_clusters(&self, data: ArrayView2<f64>, labels: &mut Array1<usize>) -> Result<(), KMeansError> {
        let centroids = self.centroids.as_ref()
            .ok_or_else(|| KMeansError::EmptyDataset)?;

        for (i, point) in data.outer_iter().enumerate() {
            let mut min_dist = f64::INFINITY;
            let mut min_cluster = 0;

            for (j, centroid) in centroids.outer_iter().enumerate() {
                let dist = euclidean_distance(point, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_cluster = j;
                }
            }

            labels[i] = min_cluster;
        }

        Ok(())
    }

    fn update_centroids(
        &self,
        data: ArrayView2<f64>,
        labels: &Array1<usize>,
        centroids: &mut Array2<f64>,
    ) -> Result<(), KMeansError> {
        centroids.fill(0.0);
        let mut counts = Array1::zeros(self.config.n_clusters);

        // Sum up all points in each cluster
        for (point, &label) in data.outer_iter().zip(labels.iter()) {
            centroids.row_mut(label).add_assign(&point);
            counts[label] += 1.0;
        }

        // Compute means
        for (k, count) in counts.iter().enumerate() {
            if *count > 0.0 {
                centroids.row_mut(k).mapv_inplace(|x| x / count);
            } else {
                // If a cluster is empty, reinitialize it to a random point
                let random_idx = rand::random::<usize>() % data.nrows();
                centroids.row_mut(k).assign(&data.row(random_idx));
            }
        }

        Ok(())
    }
}

fn euclidean_distance(a: ArrayView1<f64>, b: ArrayView1<f64>) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}
}