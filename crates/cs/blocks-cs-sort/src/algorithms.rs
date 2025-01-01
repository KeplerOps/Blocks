// Sorting algorithms
pub mod quicksort;
pub mod heapsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::heapsort::sort as heapsort;