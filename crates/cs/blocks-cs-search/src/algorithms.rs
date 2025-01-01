// Search algorithms
pub mod binary;
pub mod linear;

// Re-export commonly used functions for convenience
pub use self::binary::search as binary_search;
pub use self::binary::search_insert as binary_search_insert;
pub use self::linear::search as linear_search;