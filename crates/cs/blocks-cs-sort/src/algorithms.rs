// Sorting algorithms
pub mod quicksort;
pub mod bubblesort;

// Re-export commonly used functions for convenience
pub use self::quicksort::sort as quicksort;
pub use self::bubblesort::sort as bubblesort;