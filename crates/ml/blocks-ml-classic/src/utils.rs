use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::error::{Error, Result};

/// Validate input dimensions for supervised learning
pub fn validate_supervised_input(x: &Array2<f64>, y: &Array1<f64>) -> Result<()> {
    if x.nrows() != y.len() {
        return Err(Error::InvalidDimensions {
            expected: vec![x.nrows()],
            got: vec![y.len()],
        });
    }
    Ok(())
}

/// Split data into training and validation sets
pub fn train_test_split(
    x: &Array2<f64>,
    y: &Array1<f64>,
    test_size: f64,
) -> Result<(Array2<f64>, Array2<f64>, Array1<f64>, Array1<f64>)> {
    if !(0.0..=1.0).contains(&test_size) {
        return Err(Error::InvalidParameter(
            "test_size must be between 0 and 1".to_string(),
        ));
    }

    validate_supervised_input(x, y)?;

    let n_samples = x.nrows();
    let n_test = (n_samples as f64 * test_size).round() as usize;
    let n_train = n_samples - n_test;

    let mut indices: Vec<usize> = (0..n_samples).collect();
    indices.shuffle(&mut thread_rng());

    let (train_idx, test_idx) = indices.split_at(n_train);

    let x_train = Array2::from_shape_vec(
        (n_train, x.ncols()),
        train_idx.iter().flat_map(|&i| x.row(i).to_vec()).collect(),
    )
    .unwrap();

    let x_test = Array2::from_shape_vec(
        (n_test, x.ncols()),
        test_idx.iter().flat_map(|&i| x.row(i).to_vec()).collect(),
    )
    .unwrap();

    let y_train = Array1::from_vec(train_idx.iter().map(|&i| y[i]).collect());
    let y_test = Array1::from_vec(test_idx.iter().map(|&i| y[i]).collect());

    Ok((x_train, x_test, y_train, y_test))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_supervised_input() {
        let x = Array2::zeros((10, 5));
        let y = Array1::zeros(10);
        assert!(validate_supervised_input(&x, &y).is_ok());

        let y_wrong = Array1::zeros(5);
        assert!(validate_supervised_input(&x, &y_wrong).is_err());
    }

    #[test]
    fn test_train_test_split() {
        let x = Array2::ones((100, 5));
        let y = Array1::ones(100);
        
        let (x_train, x_test, y_train, y_test) = train_test_split(&x, &y, 0.2).unwrap();
        
        assert_eq!(x_train.nrows(), 80);
        assert_eq!(x_test.nrows(), 20);
        assert_eq!(y_train.len(), 80);
        assert_eq!(y_test.len(), 20);
        
        // Check that all values are still ones
        assert!(x_train.iter().all(|&x| (x - 1.0).abs() < f64::EPSILON));
        assert!(x_test.iter().all(|&x| (x - 1.0).abs() < f64::EPSILON));
        assert!(y_train.iter().all(|&x| (x - 1.0).abs() < f64::EPSILON));
        assert!(y_test.iter().all(|&x| (x - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_train_test_split_invalid_size() {
        let x = Array2::ones((100, 5));
        let y = Array1::ones(100);
        
        assert!(train_test_split(&x, &y, 1.5).is_err());
        assert!(train_test_split(&x, &y, -0.2).is_err());
    }
}