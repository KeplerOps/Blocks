// Sorting algorithms
pub mod quicksort;
pub mod selectionsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::selectionsort::sort as selectionsort;