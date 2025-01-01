// Sorting algorithms
pub mod quicksort;
pub mod mergesort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::mergesort::sort as mergesort;