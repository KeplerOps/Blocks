// Sorting algorithms
pub mod quicksort;
pub mod insertionsort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::insertionsort::sort as insertionsort;