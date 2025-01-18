use ndarray::{Array1, Array2, Axis};
use rand::thread_rng;
use rand::prelude::IteratorRandom;

use crate::error::{Error, Result};
use crate::traits::Unsupervised;

/// K-means clustering algorithm implementation
#[derive(Debug)]
pub struct KMeans {
    n_clusters: usize,
    max_iter: usize,
    tol: f64,
    centroids: Option<Array2<f64>>,
    inertia: Option<f64>,
}

impl Default for KMeans {
    fn default() -> Self {
        Self {
            n_clusters: 8,
            max_iter: 300,
            tol: 1e-4,
            centroids: None,
            inertia: None,
        }
    }
}

impl KMeans {
    /// Create a new KMeans instance with the specified number of clusters
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            ..Default::default()
        }
    }

    /// Set the maximum number of iterations
    pub fn max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Set the convergence tolerance
    pub fn tol(mut self, tol: f64) -> Self {
        self.tol = tol;
        self
    }

    /// Get the cluster centroids
    pub fn centroids(&self) -> Option<&Array2<f64>> {
        self.centroids.as_ref()
    }

    /// Get the inertia (sum of squared distances to closest centroid)
    pub fn inertia(&self) -> Option<f64> {
        self.inertia
    }

    /// Initialize centroids using k-means++ method
    fn initialize_centroids(&self, x: &Array2<f64>) -> Array2<f64> {
        let n_samples = x.nrows();
        let n_features = x.ncols();
        
        // Choose first centroid randomly
        let mut rng = thread_rng();
        let first_idx = (0..n_samples).choose(&mut rng).unwrap();
        let mut centroids = Array2::zeros((self.n_clusters, n_features));
        centroids
            .row_mut(0)
            .assign(&x.row(first_idx));
        
        // Choose remaining centroids
        for k in 1..self.n_clusters {
            // Calculate distances to nearest centroid for each point
            let mut min_distances = Array1::zeros(n_samples);
            for i in 0..n_samples {
                let point = x.row(i);
                let mut min_dist = f64::INFINITY;
                
                for j in 0..k {
                    let centroid = centroids.row(j);
                    let dist = point
                        .iter()
                        .zip(centroid.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>();
                    min_dist = min_dist.min(dist);
                }
                min_distances[i] = min_dist;
            }
            
            // Choose next centroid with probability proportional to distance squared
            let total_dist: f64 = min_distances.sum();
            let rand_val = rand::random::<f64>() * total_dist;
            let mut cumsum = 0.0;
            let mut chosen_idx = 0;
            
            for (i, &dist) in min_distances.iter().enumerate() {
                cumsum += dist;
                if cumsum >= rand_val {
                    chosen_idx = i;
                    break;
                }
            }
            
            centroids
                .row_mut(k)
                .assign(&x.row(chosen_idx));
        }
        
        centroids
    }

    /// Predict cluster labels for new data
    pub fn predict(&self, x: &Array2<f64>) -> Result<Array1<usize>> {
        let centroids = self.centroids.as_ref().ok_or_else(|| {
            Error::InvalidState("Model must be fitted before prediction".to_string())
        })?;

        let mut labels = Array1::zeros(x.nrows());
        
        for (i, sample) in x.rows().into_iter().enumerate() {
            let mut min_dist = f64::INFINITY;
            let mut min_cluster = 0;
            
            for (j, centroid) in centroids.rows().into_iter().enumerate() {
                let dist = sample
                    .iter()
                    .zip(centroid.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>();
                
                if dist < min_dist {
                    min_dist = dist;
                    min_cluster = j;
                }
            }
            
            labels[i] = min_cluster;
        }
        
        Ok(labels)
    }
}

impl Unsupervised for KMeans {
    fn fit(&mut self, x: &Array2<f64>) -> Result<()> {
        if x.is_empty() {
            return Err(Error::InvalidParameter("Empty input array".to_string()));
        }
        
        if self.n_clusters > x.nrows() {
            return Err(Error::InvalidParameter(
                "n_clusters cannot be greater than number of samples".to_string(),
            ));
        }

        let mut centroids = self.initialize_centroids(x);
        let mut old_centroids: Array2<f64>;
        let mut inertia = 0.0;
        
        for _ in 0..self.max_iter {
            old_centroids = centroids.clone();
            
            // Assign points to nearest centroid and calculate distances
            let mut labels = Array1::zeros(x.nrows());
            let mut distances = Array2::zeros((x.nrows(), self.n_clusters));
            
            for (i, sample) in x.rows().into_iter().enumerate() {
                let mut min_dist = f64::INFINITY;
                let mut min_cluster = 0;
                
                for (j, centroid) in centroids.rows().into_iter().enumerate() {
                    let dist = sample
                        .iter()
                        .zip(centroid.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>();
                    
                    distances[[i, j]] = dist;
                    
                    if dist < min_dist {
                        min_dist = dist;
                        min_cluster = j;
                    }
                }
                
                labels[i] = min_cluster;
            }
            
            // Update centroids
            inertia = 0.0;
            for k in 0..self.n_clusters {
                let cluster_points: Vec<_> = x
                    .rows()
                    .into_iter()
                    .zip(labels.iter())
                    .filter(|(_, &l)| l == k)
                    .map(|(row, _)| row)
                    .collect();
                
                if !cluster_points.is_empty() {
                    let new_centroid = cluster_points
                        .iter()
                        .fold(Array1::zeros(x.ncols()), |acc, row| acc + row)
                        / cluster_points.len() as f64;
                    
                    centroids.row_mut(k).assign(&new_centroid);
                    
                    // Update inertia
                    inertia += cluster_points
                        .iter()
                        .map(|point| {
                            point
                                .iter()
                                .zip(new_centroid.iter())
                                .map(|(a, b)| (a - b).powi(2))
                                .sum::<f64>()
                        })
                        .sum::<f64>();
                }
            }
            
            // Check convergence
            let centroid_shift = (&centroids - &old_centroids)
                .mapv(|x| x.powi(2))
                .sum_axis(Axis(1))
                .mapv(|x| x.sqrt())
                .iter()
                .fold(0.0_f64, |acc, &x| acc.max(x));
            
            if centroid_shift < self.tol {
                break;
            }
        }
        
        self.centroids = Some(centroids);
        self.inertia = Some(inertia);
        Ok(())
    }

    fn transform(&self, x: &Array2<f64>) -> Result<Array2<f64>> {
        let centroids = self.centroids.as_ref().ok_or_else(|| {
            Error::InvalidState("Model must be fitted before transform".to_string())
        })?;

        let mut distances = Array2::zeros((x.nrows(), self.n_clusters));

        for (i, sample) in x.rows().into_iter().enumerate() {
            for (j, centroid) in centroids.rows().into_iter().enumerate() {
                let dist = sample
                    .iter()
                    .zip(centroid.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>();
                distances[[i, j]] = dist;
            }
        }

        Ok(distances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_basic() {
        // Create a simple dataset with two clear clusters
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![
                0.0, 0.0,
                0.1, 0.1,
                0.2, 0.2,
                3.0, 3.0,
                3.1, 3.1,
                3.2, 3.2,
            ],
        )
        .unwrap();

        let mut kmeans = KMeans::new(2);
        kmeans.fit(&data).unwrap();

        // Test predictions
        let predictions = kmeans.predict(&data).unwrap();
        
        // Check that points close to each other are in the same cluster
        assert_eq!(predictions[0], predictions[1]);
        assert_eq!(predictions[1], predictions[2]);
        assert_eq!(predictions[3], predictions[4]);
        assert_eq!(predictions[4], predictions[5]);
        
        // Check that distant points are in different clusters
        assert_ne!(predictions[0], predictions[3]);
    }

    #[test]
    fn test_kmeans_empty_input() {
        let data = Array2::zeros((0, 2));
        let mut kmeans = KMeans::new(2);
        assert!(kmeans.fit(&data).is_err());
    }

    #[test]
    fn test_kmeans_too_many_clusters() {
        let data = Array2::zeros((3, 2));
        let mut kmeans = KMeans::new(4);
        assert!(kmeans.fit(&data).is_err());
    }

    #[test]
    fn test_kmeans_transform() {
        let data = Array2::from_shape_vec(
            (4, 2),
            vec![
                0.0, 0.0,
                0.1, 0.1,
                3.0, 3.0,
                3.1, 3.1,
            ],
        )
        .unwrap();

        let mut kmeans = KMeans::new(2);
        kmeans.fit(&data).unwrap();

        let distances = kmeans.transform(&data).unwrap();
        assert_eq!(distances.shape(), &[4, 2]);

        // Points should have smaller squared distance to their own cluster centroid
        let labels = kmeans.predict(&data).unwrap();
        for (i, &label) in labels.iter().enumerate() {
            assert!(distances[[i, label as usize]] < distances[[i, 1 - label as usize]]);
        }
    }

    #[test]
    fn test_kmeans_predict_without_fit() {
        let data = Array2::zeros((3, 2));
        let kmeans = KMeans::new(2);
        assert!(kmeans.predict(&data).is_err());
    }
}