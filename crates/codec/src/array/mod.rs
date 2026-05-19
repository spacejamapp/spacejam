//! Fixed-length array types with serde support for any N.

mod fixed;
mod heap;

pub use fixed::FixedArray;
pub use heap::Array;
