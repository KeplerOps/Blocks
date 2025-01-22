pub mod bfs;
pub mod binary;
pub mod dfs;
pub mod exponential;
pub mod fibonacci;
pub mod interpolation;
pub mod jump;
pub mod linear;
pub mod sublist;
pub mod ternary;

// Re-export all search algorithms
pub use bfs::*;
pub use binary::*;
pub use dfs::*;
pub use exponential::*;
pub use fibonacci::*;
pub use interpolation::*;
pub use jump::*;
pub use linear::*;
pub use sublist::*;
pub use ternary::*;
