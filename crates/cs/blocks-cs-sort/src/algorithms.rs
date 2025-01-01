// Sorting algorithms
pub mod quicksort;
pub mod countingsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::countingsort::sort as countingsort;