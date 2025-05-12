//! Visitors for the codec.

pub use {fixed::FixedBytesVisitor, vlen::VlenBytesVisitor};

mod fixed;
mod vlen;
