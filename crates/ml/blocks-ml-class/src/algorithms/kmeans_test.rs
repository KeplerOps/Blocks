#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::{arr2, Array2};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_kmeans_new_valid_config() {
        let config = KMeansConfig {
            n_clusters: 3,
            max_iterations: 100,
            convergence_threshold: 1e-4,
            random_seed: Some(42),
        };
        assert!(KMeans::new(config).is_ok());
    }

    #[test]
    fn test_kmeans_new_invalid_clusters() {
        let config = KMeansConfig {
            n_clusters: 0,
            ..Default::default()
        };
        assert!(matches!(
            KMeans::new(config).unwrap_err(),
            KMeansError::InvalidClusters
        ));
    }

    #[test]
    fn test_kmeans_new_invalid_iterations() {
        let config = KMeansConfig {
            max_iterations: 0,
            ..Default::default()
        };
        assert!(matches!(
            KMeans::new(config).unwrap_err(),
            KMeansError::InvalidMaxIterations
        ));
    }

    #[test]
    fn test_kmeans_new_invalid_threshold() {
        let config = KMeansConfig {
            convergence_threshold: -1.0,
            ..Default::default()
        };
        assert!(matches!(
            KMeans::new(config).unwrap_err(),
            KMeansError::InvalidConvergenceThreshold
        ));
    }

    #[test]
    fn test_kmeans_fit_empty_dataset() {
        let mut kmeans = KMeans::new(KMeansConfig::default()).unwrap();
        let empty_data = Array2::<f64>::zeros((0, 2));
        assert!(matches!(
            kmeans.fit(empty_data.view()),
            Err(KMeansError::EmptyDataset)
        ));
    }

    #[test]
    fn test_kmeans_predict_without_fit() {
        let kmeans = KMeans::new(KMeansConfig::default()).unwrap();
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        assert!(kmeans.predict(data.view()).is_err());
    }

    #[test]
    fn test_kmeans_centroids_without_fit() {
        let kmeans = KMeans::new(KMeansConfig::default()).unwrap();
        assert!(kmeans.centroids().is_none());
    }

    #[test]
    fn test_kmeans_fit_predict_simple_case() {
        let config = KMeansConfig {
            n_clusters: 2,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            random_seed: Some(42),
        };
        let mut kmeans = KMeans::new(config).unwrap();
        
        // Create two well-separated clusters
        let data = arr2(&[
            [0.0, 0.0],
            [0.1, 0.1],
            [0.2, 0.0],
            [10.0, 10.0],
            [10.1, 10.1],
            [10.2, 10.0],
        ]);
        
        let labels = kmeans.fit_predict(data.view()).unwrap();
        assert_eq!(labels.len(), 6);
        
        // Points in the same cluster should have the same label
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);
        
        // Points in different clusters should have different labels
        assert_ne!(labels[0], labels[3]);
    }

    #[test]
    fn test_kmeans_convergence() {
        let config = KMeansConfig {
            n_clusters: 3,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            random_seed: Some(42),
        };
        let mut kmeans = KMeans::new(config).unwrap();
        
        // Create three well-separated clusters
        let data = arr2(&[
            [0.0, 0.0], [0.1, 0.1], [0.2, 0.0],
            [10.0, 10.0], [10.1, 10.1], [10.2, 10.0],
            [20.0, 0.0], [20.1, 0.1], [20.2, 0.0],
        ]);
        
        kmeans.fit(data.view()).unwrap();
        let centroids = kmeans.centroids().unwrap();
        
        // Check number of centroids
        assert_eq!(centroids.shape(), &[3, 2]);
        
        // Verify centroids are well-separated
        for i in 0..3 {
            for j in (i + 1)..3 {
                let dist = euclidean_distance(
                    centroids.row(i).view(),
                    centroids.row(j).view(),
                );
                assert!(dist > 5.0, "Centroids {} and {} are too close", i, j);
            }
        }
    }

    #[test]
    fn test_kmeans_reproducibility() {
        let config = KMeansConfig {
            n_clusters: 2,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            random_seed: Some(42),
        };
        
        let data = arr2(&[
            [0.0, 0.0], [0.1, 0.1],
            [10.0, 10.0], [10.1, 10.1],
        ]);
        
        let mut kmeans1 = KMeans::new(config.clone()).unwrap();
        let mut kmeans2 = KMeans::new(config).unwrap();
        
        let labels1 = kmeans1.fit_predict(data.view()).unwrap();
        let labels2 = kmeans2.fit_predict(data.view()).unwrap();
        
        assert_eq!(labels1, labels2);
        
        let centroids1 = kmeans1.centroids().unwrap();
        let centroids2 = kmeans2.centroids().unwrap();
        
        assert_relative_eq!(centroids1, centroids2, epsilon = 1e-10);
    }

    #[test]
    fn test_kmeans_inertia() {
        let config = KMeansConfig {
            n_clusters: 2,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            random_seed: Some(42),
        };
        let mut kmeans = KMeans::new(config).unwrap();
        
        let data = arr2(&[
            [0.0, 0.0], [0.0, 0.0],  // Perfect cluster 1
            [1.0, 1.0], [1.0, 1.0],  // Perfect cluster 2
        ]);
        
        kmeans.fit(data.view()).unwrap();
        let inertia = kmeans.inertia(data.view()).unwrap();
        
        // Since points in each cluster are identical, inertia should be 0
        assert_relative_eq!(inertia, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_kmeans_with_random_data() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let n_samples = 1000;
        let n_features = 2;
        let n_clusters = 5;
        
        // Generate random data
        let data = Array2::random_using((n_samples, n_features), rand::distributions::Uniform::new(-10.0, 10.0), &mut rng);
        
        let config = KMeansConfig {
            n_clusters,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            random_seed: Some(42),
        };
        let mut kmeans = KMeans::new(config).unwrap();
        
        // Ensure fitting works without errors
        kmeans.fit(data.view()).unwrap();
        
        // Check that we have the correct number of clusters
        assert_eq!(kmeans.centroids().unwrap().shape(), &[n_clusters, n_features]);
        
        // Predict clusters for the same data
        let labels = kmeans.predict(data.view()).unwrap();
        assert_eq!(labels.len(), n_samples);
        
        // Check that all cluster assignments are valid
        assert!(labels.iter().all(|&x| x < n_clusters));
    }

    fn euclidean_distance(a: ArrayView1<f64>, b: ArrayView1<f64>) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}