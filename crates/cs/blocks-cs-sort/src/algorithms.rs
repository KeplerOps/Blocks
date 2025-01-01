// Sorting algorithms
pub mod quicksort;
pub mod bucketsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::bucketsort::sort as bucketsort;