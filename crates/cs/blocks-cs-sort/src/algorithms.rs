// Sorting algorithms
pub mod quicksort;
pub mod shellsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::shellsort::sort as shellsort;