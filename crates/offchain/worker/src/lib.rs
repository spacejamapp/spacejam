//! Spacejam work package builder
//! Work package computation

pub use {
    d3l::{DataLake, InMemoryDataLake, Specifier},
    worker::Worker,
};

mod bundle;
pub mod d3l;
mod worker;
